use super::Texture;

pub fn std_shared_ptr_texture_assign(slot: &mut Option<Texture>, value: Option<Texture>) {
    *slot = value;
}
