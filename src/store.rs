use std::collections::HashMap;
use std::sync::Mutex;

pub struct Store {
    pub data: Mutex<HashMap<String, String>>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            data: Mutex::new(HashMap::new()),
        }
    }
}