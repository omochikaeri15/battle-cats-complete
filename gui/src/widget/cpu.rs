mod raster;

use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use iced::advanced::image::Handle;
use iced::widget::canvas::{self, Canvas, Geometry};
use iced::widget::shader::{self, Shader};
use iced::widget::stack;
use iced::{mouse, Element, Event, Length, Rectangle, Theme};
use image::RgbaImage;
use rustc_hash::FxHasher;
use tracing::info;

pub(crate) use raster::{Alpha, Vertex};

static SOFTWARE: AtomicBool = AtomicBool::new(false);

pub(crate) fn software() -> bool {
    SOFTWARE.load(Ordering::Relaxed)
}

fn witness(renderer: &iced::Renderer) {
    let drawn_in_software = matches!(renderer, iced_renderer::fallback::Renderer::Secondary(_));

    if drawn_in_software && !SOFTWARE.swap(true, Ordering::Relaxed) {
        info!("The app is drawn by the software renderer, so custom viewers are rasterized on the CPU");
    }
}

#[derive(Clone)]
pub(crate) enum Pixels {
    Shared(Arc<[u8]>),
    Image(Arc<RgbaImage>),
}

impl Pixels {
    fn bytes(&self) -> &[u8] {
        match self {
            Self::Shared(pixels) => pixels,
            Self::Image(image) => image.as_raw(),
        }
    }

    fn identity(&self) -> usize {
        match self {
            Self::Shared(pixels) => pixels.as_ptr() as usize,
            Self::Image(image) => Arc::as_ptr(image) as usize,
        }
    }
}

#[derive(Clone)]
pub(crate) struct Sheet {
    pub width: u32,
    pub height: u32,
    pub pixels: Pixels,
}

pub(crate) struct Layer {
    pub sheet: Option<Sheet>,
    pub blend: u8,
    pub alpha: Alpha,
    pub range: Range<usize>,
}

#[derive(Default)]
pub(crate) struct Plan {
    pub vertices: Vec<Vertex>,
    pub layers: Vec<Layer>,
}

impl Plan {
    pub(crate) fn render(&self, width: u32, height: u32, clear: [u8; 4]) -> Vec<u8> {
        let mut pixels = clear.repeat(width as usize * height as usize);
        let batches: Vec<raster::Batch<'_>> = self
            .layers
            .iter()
            .map(|layer| raster::Batch {
                texture: layer.sheet.as_ref().map(|sheet| raster::Texture { width: sheet.width, height: sheet.height, pixels: sheet.pixels.bytes() }),
                blend: layer.blend,
                alpha: layer.alpha,
                range: layer.range.clone(),
            })
            .collect();

        raster::draw(&mut pixels, width, height, &self.vertices, &batches);

        pixels
    }

    fn key(&self, width: u32, height: u32) -> u64 {
        let mut hasher = FxHasher::default();

        (width, height).hash(&mut hasher);
        bytemuck::cast_slice::<Vertex, u8>(&self.vertices).hash(&mut hasher);

        for layer in &self.layers {
            (layer.blend, layer.alpha == Alpha::Carried, layer.range.start, layer.range.end).hash(&mut hasher);
            layer.sheet.as_ref().map(|sheet| (sheet.width, sheet.height, sheet.pixels.identity())).hash(&mut hasher);
        }

        hasher.finish()
    }
}

pub(crate) trait Rasterize {
    fn plan(&self, bounds: Rectangle) -> Option<Plan>;
}

#[derive(Default)]
pub(crate) struct Held<S> {
    inner: S,
    painted: RefCell<Option<(u64, Handle)>>,
}

struct Soft<P> {
    program: P,
}

impl<M, P> canvas::Program<M> for Soft<P>
where
    P: shader::Program<M>,
    P::Primitive: Rasterize,
{
    type State = Held<P::State>;

    fn update(&self, state: &mut Self::State, event: &Event, bounds: Rectangle, cursor: mouse::Cursor) -> Option<canvas::Action<M>> {
        self.program.update(&mut state.inner, event, bounds, cursor)
    }

    fn draw(&self, state: &Self::State, renderer: &iced::Renderer, _theme: &Theme, bounds: Rectangle, cursor: mouse::Cursor) -> Vec<Geometry> {
        witness(renderer);

        let (width, height) = (bounds.width.round() as u32, bounds.height.round() as u32);

        if width == 0 || height == 0 {
            return Vec::new();
        }

        let Some(plan) = self.program.draw(&state.inner, cursor, bounds).plan(bounds) else {
            return Vec::new();
        };
        let key = plan.key(width, height);
        let mut painted = state.painted.borrow_mut();

        if painted.as_ref().is_none_or(|(held, _)| *held != key) {
            let mut pixels = plan.render(width, height, [0; 4]);

            raster::straighten(&mut pixels);
            *painted = Some((key, Handle::from_rgba(width, height, pixels)));
        }

        let Some((_, handle)) = painted.as_ref() else {
            return Vec::new();
        };
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        frame.draw_image(Rectangle::with_size(bounds.size()), handle);

        vec![frame.into_geometry()]
    }

    fn mouse_interaction(&self, state: &Self::State, bounds: Rectangle, cursor: mouse::Cursor) -> mouse::Interaction {
        self.program.mouse_interaction(&state.inner, bounds, cursor)
    }
}

struct Witness;

impl<M> canvas::Program<M> for Witness {
    type State = ();

    fn draw(&self, _state: &(), renderer: &iced::Renderer, _theme: &Theme, _bounds: Rectangle, _cursor: mouse::Cursor) -> Vec<Geometry> {
        witness(renderer);

        Vec::new()
    }
}

pub(crate) fn shader<'a, M, P>(program: P) -> Element<'a, M>
where
    M: 'a,
    P: shader::Program<M> + 'a,
    P::Primitive: Rasterize,
{
    if software() {
        return Canvas::new(Soft { program }).width(Length::Fill).height(Length::Fill).into();
    }

    stack![
        Shader::new(program).width(Length::Fill).height(Length::Fill),
        Canvas::new(Witness).width(Length::Fill).height(Length::Fill),
    ]
    .into()
}
