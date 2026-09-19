#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Texture {
    pub id: u64,
    pub width: i32,
    pub height: i32,
}

#[derive(Clone, Copy)]
pub enum FormatArg<'a> {
    Int(i32),
    Text(&'a [u8]),
}

pub trait TextRenderer {
    fn text_texture(&mut self, text: &[u8], font: &[u8], size: i32, align: i32, width: i32) -> Texture;
    fn format(&mut self, pattern: &[u8], args: &[&[u8]]) -> Vec<u8>;
    fn substitute(&mut self, text: &[u8], tokens: &[(&[u8], &[u8])]) -> Vec<u8>;
    fn format_args(&mut self, pattern: &[u8], args: &[FormatArg<'_>]) -> Vec<u8>;
    fn stage_name(&mut self, map_type: i32, map_index: i32, stage: i32) -> Vec<u8>;
}

pub fn get_text_texture(cache: &mut dyn TextRenderer, text: &[u8], font: &[u8], size: i32, align: i32, width: i32) -> Texture {
    cache.text_texture(text, font, size, align, width)
}
