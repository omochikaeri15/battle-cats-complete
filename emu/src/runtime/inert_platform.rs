use crate::engine::Platform;

pub struct InertPlatform;

impl Platform for InertPlatform {
    fn set_keep_awake(&mut self, _awake: bool) {}
}
