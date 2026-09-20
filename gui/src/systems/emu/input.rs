use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy, Debug)]
pub enum Touch {
    Moved { x: i32, y: i32 },
    Pressed { x: i32, y: i32 },
    Released,
}

pub type TouchQueue = Rc<RefCell<Vec<Touch>>>;

pub fn queue() -> TouchQueue {
    Rc::new(RefCell::new(Vec::new()))
}
