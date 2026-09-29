use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};
use tracing::warn;

use crate::common::dirs;

pub fn save<T: Serialize>(filename: &str, data: &T) -> io::Result<()> {
    let directory = dirs::config().ok_or_else(|| Error::other("Config directory unavailable"))?;
    save_in(&directory, filename, data)
}

pub fn save_state<T: Serialize>(filename: &str, data: &T) -> io::Result<()> {
    let directory = dirs::state().ok_or_else(|| Error::other("State directory unavailable"))?;
    save_in(&directory, filename, data)
}

pub fn load_state<T: DeserializeOwned>(filename: &str) -> Option<T> {
    load_in(&dirs::state()?, filename)
}

pub fn config_file(filename: &str) -> Option<PathBuf> {
    Some(dirs::config()?.join(filename))
}

pub fn state_file(filename: &str) -> Option<PathBuf> {
    Some(dirs::state()?.join(filename))
}

pub fn salvage<T: Serialize + DeserializeOwned + Default>(path: Option<PathBuf>) -> T {
    let Some(path) = path else {
        return T::default();
    };

    match fs::read_to_string(&path) {
        Ok(data) => recover(&path, &data),
        Err(err) if err.kind() == ErrorKind::NotFound => T::default(),
        Err(err) => {
            warn!("Failed to read {}, starting from defaults: {}", path.display(), err);
            T::default()
        }
    }
}

fn recover<T: Serialize + DeserializeOwned + Default>(path: &Path, data: &str) -> T {
    let saved = match serde_json::from_str::<Value>(data) {
        Ok(saved) => saved,
        Err(err) => {
            warn!("{} is not valid JSON, starting from defaults: {}", path.display(), err);
            return T::default();
        }
    };

    if let Ok(parsed) = T::deserialize(&saved) {
        return parsed;
    }

    let (Value::Object(saved), Ok(mut merged)) = (saved, serde_json::to_value(T::default())) else {
        warn!("{} does not match the current layout, starting from defaults", path.display());
        return T::default();
    };

    warn!("{} does not fully match the current layout, keeping every section that still reads", path.display());
    graft::<T>(&mut merged, "", saved);

    T::deserialize(&merged).unwrap_or_default()
}

fn graft<T: DeserializeOwned>(root: &mut Value, pointer: &str, saved: Map<String, Value>) {
    for (key, value) in saved {
        let Some(previous) = root.pointer_mut(pointer).and_then(Value::as_object_mut).map(|parent| parent.insert(key.clone(), value)) else {
            return;
        };

        if T::deserialize(&*root).is_ok() {
            continue;
        }

        let Some(parent) = root.pointer_mut(pointer).and_then(Value::as_object_mut) else {
            return;
        };

        let rejected = match previous {
            Some(previous) => parent.insert(key.clone(), previous),
            None => parent.remove(&key),
        };
        let nested = matches!(parent.get(&key), Some(Value::Object(_)));

        if let Some(Value::Object(rejected)) = rejected && nested {
            graft::<T>(root, &format!("{pointer}/{}", key.replace('~', "~0").replace('/', "~1")), rejected);
        } else {
            warn!("Dropped unreadable saved state at {}/{}", pointer, key);
        }
    }
}

fn save_in<T: Serialize>(directory: &Path, filename: &str, data: &T) -> io::Result<()> {
    let path = directory.join(filename);

    let json = serde_json::to_string_pretty(data).map_err(Error::other)?;
    let tmp_path = super::hidden_temp(&path);
    fs::write(&tmp_path, json)?;
    fs::rename(&tmp_path, &path)
}

fn load_in<T: DeserializeOwned>(directory: &Path, filename: &str) -> Option<T> {
    let path = directory.join(filename);

    if path.exists()
        && let Ok(data) = fs::read_to_string(&path)
        && let Ok(parsed) = serde_json::from_str::<T>(&data) {
        return Some(parsed);
    }
    None
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
    #[serde(default)]
    struct Inner {
        agreement: String,
        mode: u8,
    }

    #[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
    #[serde(default)]
    struct Outer {
        sandbox: Inner,
        count: u32,
    }

    #[test]
    fn one_bad_field_keeps_the_rest() {
        // mode went from a string to a number in a later version; the agreement must survive that
        let old = r#"{"sandbox":{"agreement":"abc","mode":"Wide"},"count":7}"#;
        let loaded: Outer = recover(Path::new("state.json"), old);

        assert_eq!(loaded, Outer { sandbox: Inner { agreement: "abc".into(), mode: 0 }, count: 7 });
    }
}
