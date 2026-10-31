use std::cell::RefCell;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Touch {
    Moved { x: i32, y: i32 },
    Pressed { x: i32, y: i32 },
    Released,
    Pinched { spread: i32 },
}

pub type TouchQueue = Rc<RefCell<Vec<Touch>>>;

pub fn queue() -> TouchQueue {
    Rc::new(RefCell::new(Vec::new()))
}
