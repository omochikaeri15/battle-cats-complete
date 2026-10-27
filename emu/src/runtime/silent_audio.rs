use std::rc::Rc;

use crate::engine::{AudioHost, AudioSource, SoundData};

pub struct SilentAudio;

impl AudioHost for SilentAudio {
    fn active_sources(&mut self) -> i32 {
        0
    }

    fn source_capacity(&mut self) -> i32 {
        0
    }

    fn new_buffer_source(&mut self, _data: &Rc<dyn SoundData>) -> Option<Rc<dyn AudioSource>> {
        None
    }

    fn new_stream_source(&mut self, _path: &[u8], _source_type: i32) -> Option<Rc<dyn AudioSource>> {
        None
    }

    fn file_exists(&mut self, _path: &[u8]) -> bool {
        false
    }

    fn files_dir(&mut self) -> Vec<u8> {
        Vec::new()
    }

    fn remove_file(&mut self, _path: &[u8]) -> bool {
        false
    }

    fn load_sound_data(&mut self, _path: &[u8]) -> Option<Rc<dyn SoundData>> {
        None
    }
}
