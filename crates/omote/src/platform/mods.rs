use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ModCommand {
    Held,
    Upload { name: String, bytes: Vec<u8> },
    Enable,
    Disable,
    Delete,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModInfo {
    pub name: String,
    pub title: String,
    pub author: String,
    pub version: String,
    pub package: String,
    pub description: String,
    pub files: usize,
    pub bytes: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Mounted {
    pub names: Vec<Box<str>>,
    pub withheld: Vec<Box<str>>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ModState {
    pub held: Option<ModInfo>,
    pub mounted: Option<Mounted>,
}
