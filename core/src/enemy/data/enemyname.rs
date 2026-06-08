use std::fs;
use std::path::Path;
use nyanko::enemy::unit::EnemyName;

pub fn load(lang_dir: &Path, priority: &[String]) -> Vec<String> {
    let mut final_names: Vec<String> = Vec::new();
    let base_dir = lang_dir.join("Enemyname");

    for file_path in crate::global::resolver::get(&base_dir, ["Enemyname.tsv"], priority) {
        let Ok(bytes) = fs::read(&file_path) else { continue; };
        let Ok(parsed_names) = EnemyName::parse_all(bytes) else { continue; };

        for (index, enemy) in parsed_names.into_iter().enumerate() {
            if index >= final_names.len() {
                final_names.push(enemy.name.unwrap_or_default());
                continue;
            }

            if final_names[index].is_empty() {
                if let Some(valid_name) = enemy.name {
                    final_names[index] = valid_name;
                }
            }
        }
    }

    final_names
}

pub fn load_single(lang_dir: &Path, priority: &[String], id: usize) -> Option<String> {
    let base_dir = lang_dir.join("Enemyname");

    for file_path in crate::global::resolver::get(&base_dir, ["Enemyname.tsv"], priority) {
        let Ok(bytes) = fs::read(&file_path) else { continue; };

        if let Ok(Some(enemy)) = EnemyName::parse(bytes, id) {
            if let Some(valid_name) = enemy.name {
                return Some(valid_name);
            }
        }
    }

    None
}