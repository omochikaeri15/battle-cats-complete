use super::Texture;

pub fn std_shared_ptr_texture_reset(slot: &mut Option<Texture>) {
    *slot = None;
}
