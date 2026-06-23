use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

pub struct StoreEntry {
    pub value: String,
    pub expires_at: Option<Instant>,
}

pub struct Store {
    pub data: Mutex<HashMap<String, StoreEntry>>,
}


impl Store {
    pub fn new() -> Self {
        Store {
            data: Mutex::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let mut data = self.data.lock().unwrap();

        if let Some(entry) = data.get(key) {
            if Self::is_expired(entry) {
                data.remove(key);
                return None;
            } 
            return Some(entry.value.clone());
        }

        None
    }

    pub fn set(&self, key: String, value: String) {
        let mut data = self.data.lock().unwrap();
        data.insert(key, StoreEntry{value, expires_at: None});
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
            if let Some(entry) = data.get(key) {
                if !Self::is_expired(entry) {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn keys(&self) -> Vec<String> {
        let data = self.data.lock().unwrap();
        data.iter()
            .filter(|(_, entry)| !Self::is_expired(entry))
            .map(|(k, _)| k.clone())
            .collect()
    }

    fn is_expired(entry: &StoreEntry) -> bool {
        match entry.expires_at {
            Some(t) => Instant::now() >= t,
            None => false,
        }
    }
}
