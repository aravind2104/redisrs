use std::collections::{HashMap, VecDeque, HashSet};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::time;
use std::sync::Arc;

#[derive(Clone)]
pub enum StoreValue {
    StringVal(String),
    ListVal(VecDeque<String>),
    HashVal(HashMap<String, String>),
    SetVal(HashSet<String>),
}
pub struct StoreEntry {
    pub value: StoreValue,
    pub expires_at: Option<Instant>,
}

pub struct Store {
    pub data: Mutex<HashMap<String, StoreEntry>>,
}

impl Store {
    // Creates a new instance of the Store with an empty HashMap.
    pub fn new() -> Self {
        Store {
            data: Mutex::new(HashMap::new()),
        }
    }

    // Retrieves the value associated with the given key, if it exists and is not expired. If the key is expired, it will be removed from the store and None will be returned.
    pub fn get(&self, key: &str) -> Result<Option<String>, String> {
        let mut data = self.data.lock().unwrap();

        if let Some(entry) = data.get(key) {
            if Self::is_expired(entry) {
                data.remove(key);
                return Ok(None);
            }
            match &entry.value {
                StoreValue::StringVal(s) => {
                    return Ok(Some(s.clone()));
                }
                _ => {
                    return Err(format!("WRONGTYPE Operation against a key holding the wrong kind of value"));
                }
            }
        }

        Ok(None)
    }

    // Sets the value for the given key in the store. If the key already exists, its value will be updated. The expiration time is set to None, meaning the key will not expire unless explicitly set later.
    pub fn set(&self, key: String, value: String) {
        let mut data = self.data.lock().unwrap();
        data.insert(
            key,
            StoreEntry {
                value: StoreValue::StringVal(value),
                expires_at: None,
            },
        );
    }

    //Sets the value for the given key in the store with an expiration time in seconds. If the key already exists, its value and expiration time will be updated.
    pub fn set_with_expiry(&self, key: String, value: String, duration: Duration) {
        let mut data = self.data.lock().unwrap();
        let expires_at = Instant::now() + duration;
        data.insert(
            key,
            StoreEntry {
                value: StoreValue::StringVal(value),
                expires_at: Some(expires_at),
            },
        );
    }

    // Removes the specified keys from the store and returns the number of keys removed.
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

    // Checks if the specified keys exist in the store and returns the count of existing keys. Expired keys are not counted as existing.
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

    //Returns a vector of all keys in the store that are not expired.
    pub fn keys(&self) -> Vec<String> {
        let data = self.data.lock().unwrap();
        data.iter()
            .filter(|(_, entry)| !Self::is_expired(entry))
            .map(|(k, _)| k.clone())
            .collect()
    }

    //To Set an expiration time for a key in seconds. Returns true if the key exists and the expiration was set, false otherwise.
    pub fn expire(&self, key: &str, seconds: u64) -> bool {
        let mut data = self.data.lock().unwrap();
        if let Some(entry) = data.get_mut(key) {
            entry.expires_at = Some(Instant::now() + Duration::from_secs(seconds));

            true
        } else {
            false
        }
    }

    //To remove the expiration from a key, making it persistent. Returns true if the key exists and the expiration was removed, false otherwise.
    pub fn persist(&self, key: &str) -> bool {
        let mut data = self.data.lock().unwrap();
        if let Some(entry) = data.get_mut(key) {
            let had_expiry = entry.expires_at.is_some();

            entry.expires_at = None;

            had_expiry
        } else {
            false
        }
    }

    pub fn ttl(&self, key: &str) -> i64 {
        let mut data = self.data.lock().unwrap();

        if let Some(entry) = data.get(key) {
            if Self::is_expired(entry) {
                data.remove(key);
                return -2; // Key does not exist(expired)
            } else {
                if let Some(expiry) = entry.expires_at {
                    let ttl = expiry.saturating_duration_since(Instant::now());
                    return ttl.as_secs() as i64;
                } else {
                    return -1; // Key exists but has no expiry
                }
            }
        } else {
            return -2; // Key does not exist
        }
    }

    pub async fn active_expiry_task(store: Arc<Store>){
        loop{
            time::sleep(Duration::from_secs(1)).await;
            let mut data = store.data.lock().unwrap();
            let now = Instant::now();
            data.retain(|_,entry|{
                match entry.expires_at{
                    Some(expires_at) => expires_at > now,
                    None => true,
                }
            });
        }
    }

    //Helper function to check if a StoreEntry is expired
    fn is_expired(entry: &StoreEntry) -> bool {
        match entry.expires_at {
            Some(t) => Instant::now() >= t,
            None => false,
        }
    }
}
