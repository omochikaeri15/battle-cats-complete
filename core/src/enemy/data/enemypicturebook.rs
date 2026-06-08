use std::fs;
use std::path::Path;
use nyanko::enemy::unit::EnemyPictureBook;

pub fn load(lang_dir: &Path, priority: &[String]) -> Vec<Vec<String>> {
    let mut final_descriptions: Vec<Vec<String>> = Vec::new();
    let base_dir = lang_dir.join("EnemyPictureBook");

    for file_path in crate::global::resolver::get(&base_dir, ["EnemyPictureBook.csv"], priority) {
        let Ok(bytes) = fs::read(&file_path) else { continue; };
        let Ok(parsed_books) = EnemyPictureBook::parse_all(bytes) else { continue; };

        for (index, enemy) in parsed_books.into_iter().enumerate() {
            if index >= final_descriptions.len() {
                final_descriptions.push(enemy.description.unwrap_or_default());
                continue;
            }

            if final_descriptions[index].is_empty() {
                if let Some(valid_desc) = enemy.description {
                    final_descriptions[index] = valid_desc;
                }
            }
        }
    }

    final_descriptions
}

pub fn load_single(lang_dir: &Path, priority: &[String], id: usize) -> Option<Vec<String>> {
    let base_dir = lang_dir.join("EnemyPictureBook");

    for file_path in crate::global::resolver::get(&base_dir, ["EnemyPictureBook.csv"], priority) {
        let Ok(bytes) = fs::read(&file_path) else { continue; };

        if let Ok(Some(enemy)) = EnemyPictureBook::parse(bytes, id) {
            if let Some(valid_desc) = enemy.description {
                return Some(valid_desc);
            }
        }
    }

    None
}