use std::any::Any;
use std::rc::Rc;

use crate::Fault;

use super::AppContext;

pub trait SoundData {
    fn as_any(&self) -> &dyn Any;
}

pub trait AudioSource {
    fn is_playing(&self) -> bool;
    fn is_stopped(&self) -> bool;
    fn play(&self);
    fn set_loop(&self, looping: bool);
    fn set_volume(&self, volume: f32);
    fn stop(&self);
    fn priority(&self) -> i32;
}

pub trait AudioHost {
    fn active_sources(&mut self) -> i32;
    fn source_capacity(&mut self) -> i32;
    fn new_buffer_source(&mut self, data: &Rc<dyn SoundData>) -> Option<Rc<dyn AudioSource>>;
    fn new_stream_source(&mut self, path: &[u8], source_type: i32) -> Option<Rc<dyn AudioSource>>;
    fn file_exists(&mut self, path: &[u8]) -> bool;
    fn files_dir(&mut self) -> Vec<u8>;
    fn remove_file(&mut self, path: &[u8]) -> bool;
    fn load_sound_data(&mut self, path: &[u8]) -> Option<Rc<dyn SoundData>>;
}

pub struct SoundManager {
    pub count: i32,
    pub kinds: Vec<i32>,
    pub priorities: Vec<i32>,
    pub caf_lookup_mode: i32,
    pub sound_data: Vec<Option<Rc<dyn SoundData>>>,
    pub sources: Vec<Option<Rc<dyn AudioSource>>>,
    pub bgm_source: Option<Rc<dyn AudioSource>>,
    pub bgm_muted: u8,
    pub se_muted: u8,
    pub looping: Vec<bool>,
    pub bgm_duck: i32,
    pub volumes: Vec<i32>,
    pub bgm_volume: i32,
    pub se_volume: i32,
    pub paused: u8,
}

impl Default for SoundManager {
    fn default() -> Self {
        Self {
            count: 0,
            kinds: Vec::new(),
            priorities: Vec::new(),
            caf_lookup_mode: 0,
            sound_data: Vec::new(),
            sources: Vec::new(),
            bgm_source: None,
            bgm_muted: 0,
            se_muted: 0,
            looping: Vec::new(),
            bgm_duck: 0x64,
            volumes: Vec::new(),
            bgm_volume: 0x64,
            se_volume: 0x64,
            paused: 0,
        }
    }
}

pub fn sound_manager(ctx: &mut AppContext) -> Result<&mut SoundManager, Fault> {
    Ok(&mut ctx.sound_manager)
}
