use std::fs;
use std::path::Path;
use nyanko::enemy::unit::Battle;

pub fn load_all(dir: &Path, filename: &str, priority: &[String]) -> Option<Vec<Battle>> {
    let path = crate::global::resolver::get(dir, [filename], priority).into_iter().next()?;
    let bytes = fs::read(path).ok()?;
    Battle::parse_all(bytes).ok()
}

pub fn load_single(dir: &Path, filename: &str, priority: &[String], id: usize) -> Option<Battle> {
    let path = crate::global::resolver::get(dir, [filename], priority).into_iter().next()?;
    let bytes = fs::read(path).ok()?;
    Battle::parse(bytes, id).ok().flatten()
}