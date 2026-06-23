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

    pub fn get(&self, key:&str) -> Option<String> {
        let data = self.data.lock().unwrap();
        data.get(key).cloned()
    }

    pub fn set(&self, key: String, value: String) {
        let mut data = self.data.lock().unwrap();
        data.insert(key, value);
    }

    pub fn del(&self, keys: &[String]) -> i64 {
        let mut data = self.data.lock().unwrap();
        let mut count = 0;
        for key in keys {
            if data.remove(key).is_some() {
                count += 1;
            }
        }
        count
    }

    pub fn exists(&self, keys: &[String]) -> i64 {
        let data = self.data.lock().unwrap();
        let mut count = 0;
        for key in keys {
            if data.contains_key(key) {
                count += 1;
            }
        }
        count
    }

    pub fn keys(&self) -> Vec<String> {
        let data = self.data.lock().unwrap();
        data.keys().cloned().collect()
    }
}