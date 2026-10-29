use super::EventItemStore;

pub fn is_score_stage(store: Option<&EventItemStore>) -> bool {
    match store {
        Some(store) => store.score_stage,
        None => false,
    }
}
