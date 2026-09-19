#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Texture {
    pub id: u64,
    pub width: i32,
    pub height: i32,
}

pub trait TextRenderer {
    fn text_texture(&mut self, text: &[u8], font: &[u8], size: i32, align: i32, width: i32) -> Texture;
}

pub fn get_text_texture(cache: &mut dyn TextRenderer, text: &[u8], font: &[u8], size: i32, align: i32, width: i32) -> Texture {
    cache.text_texture(text, font, size, align, width)
}
