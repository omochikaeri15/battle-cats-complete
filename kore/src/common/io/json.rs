use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};
use tracing::warn;

use crate::common::dirs;

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

    let Some(merged) = mend::<T>(saved) else {
        warn!("{} does not match the current layout, starting from defaults", path.display());
        return T::default();
    };

    warn!("{} does not fully match the current layout, keeping every section that still reads", path.display());
    T::deserialize(&merged).unwrap_or_default()
}

fn mend<T: Serialize + DeserializeOwned + Default>(saved: Value) -> Option<Value> {
    if T::deserialize(&saved).is_ok() {
        return Some(saved);
    }

    let (Value::Object(saved), Ok(mut merged)) = (saved, serde_json::to_value(T::default())) else {
        return None;
    };

    graft::<T>(&mut merged, "", saved);
    Some(merged)
}

pub fn persist<T: Serialize + DeserializeOwned + Default>(path: Option<PathBuf>, data: &T) -> io::Result<()> {
    let path = path.ok_or_else(|| Error::other("Save directory unavailable"))?;
    let fresh = serde_json::to_value(data).map_err(Error::other)?;

    #[cfg(debug_assertions)]
    let fresh = carry::<T>(&path, fresh);

    write(&path, &fresh)
}

#[cfg(debug_assertions)]
fn carry<T: Serialize + DeserializeOwned + Default>(path: &Path, mut fresh: Value) -> Value {
    let Some(held) = fs::read_to_string(path).ok().and_then(|data| serde_json::from_str::<Value>(&data).ok()).and_then(mend::<T>) else {
        return fresh;
    };

    let mut unknown = Vec::new();
    let read = serde_ignored::deserialize::<_, _, T>(&held, |ignored| {
        if let serde_ignored::Path::Map { parent, key } = ignored
            && let Some(parent) = pointer(parent)
        {
            unknown.push((parent, key));
        }
    });

    if let Err(err) = read {
        warn!("Could not trace unknown keys in {}: {}", path.display(), err);
        return fresh;
    }

    for (parent, key) in unknown {
        let kept = held.pointer(&format!("{parent}/{}", escape(&key)));

        if let (Some(kept), Some(Value::Object(slot))) = (kept, fresh.pointer_mut(&parent)) {
            slot.entry(key).or_insert_with(|| kept.clone());
        }
    }

    fresh
}

#[cfg(debug_assertions)]
fn pointer(path: &serde_ignored::Path) -> Option<String> {
    match path {
        serde_ignored::Path::Root => Some(String::new()),
        serde_ignored::Path::Seq { parent, index } => pointer(parent).map(|parent| format!("{parent}/{index}")),
        serde_ignored::Path::Map { parent, key } => pointer(parent).map(|parent| format!("{parent}/{}", escape(key))),
        serde_ignored::Path::Some { parent } | serde_ignored::Path::NewtypeStruct { parent } => pointer(parent),
        serde_ignored::Path::NewtypeVariant { .. } => None,
    }
}

fn escape(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
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
            graft::<T>(root, &format!("{pointer}/{}", escape(&key)), rejected);
        } else {
            warn!("Dropped unreadable saved state at {}/{}", pointer, key);
        }
    }
}

fn save_in<T: Serialize>(directory: &Path, filename: &str, data: &T) -> io::Result<()> {
    write(&directory.join(filename), data)
}

fn write<T: Serialize>(path: &Path, data: &T) -> io::Result<()> {
    let json = serde_json::to_string_pretty(data).map_err(Error::other)?;
    let tmp_path = super::hidden_temp(path);
    fs::write(&tmp_path, json)?;
    fs::rename(&tmp_path, path)
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

    #[cfg(debug_assertions)]
    #[derive(Serialize, Deserialize, Default)]
    #[serde(default)]
    struct Held {
        count: u32,
        popups: std::collections::BTreeMap<String, u32>,
    }

    #[cfg(debug_assertions)]
    #[test]
    fn debug_saves_carry_unknown_keys_but_not_removed_entries() {
        let path = std::env::temp_dir().join(format!("bcc-carry-{}.json", std::process::id()));
        fs::write(&path, r#"{"count":1,"popups":{"old":3},"newer":{"x":1}}"#).unwrap();

        // "newer" belongs to a build this one doesn't know; "old" was closed on purpose
        persist(Some(path.clone()), &Held { count: 2, popups: Default::default() }).unwrap();
        let written: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        fs::remove_file(&path).unwrap();

        assert_eq!(written, serde_json::json!({"count":2,"popups":{},"newer":{"x":1}}));
    }

    #[test]
    fn one_bad_field_keeps_the_rest() {
        // mode went from a string to a number in a later version; the agreement must survive that
        let old = r#"{"sandbox":{"agreement":"abc","mode":"Wide"},"count":7}"#;
        let loaded: Outer = recover(Path::new("state.json"), old);

        assert_eq!(loaded, Outer { sandbox: Inner { agreement: "abc".into(), mode: 0 }, count: 7 });
    }
}
