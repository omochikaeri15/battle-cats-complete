use std::fs;
use std::path::Path;
use std::collections::HashMap;
use crate::cat::paths;
use nyanko::cat::unit::UnitEvolve;

pub fn load(cats_directory: &Path, priority: &[String]) -> HashMap<u32, UnitEvolve> {
    let mut final_map: HashMap<u32, UnitEvolve> = HashMap::new();
    let base_directory = cats_directory.join(paths::DIR_UNIT_EVOLVE);
    
    for file_path in crate::global::resolver::get(&base_directory, ["unitevolve.csv"], priority) {
        if let Ok(bytes) = fs::read(&file_path) {
            if let Ok(parsed_map) = UnitEvolve::parse(&bytes) {
                for (cat_id, parsed_evolve) in parsed_map {
                    let entry = final_map.entry(cat_id).or_default();

                    for i in 0..4 {
                        if entry.texts[i].is_none() && parsed_evolve.texts[i].is_some() {
                            entry.texts[i] = parsed_evolve.texts[i].clone();
                        }
                    }
                }
            }
        }
    }

    final_map
}