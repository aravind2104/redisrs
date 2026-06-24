use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::time;

const WRONG_TYPE: &str = "WRONGTYPE Operation against a key holding the wrong kind of value";

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
                    return Err(WRONG_TYPE.to_string());
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

    //This function is intended to implement the LPUSH command, which adds one or more values to the head of a list stored at a given key. If the key does not exist, it should create a new list. If the key exists but is not a list, it should return an error. The function should return the length of the list after the push operation.
    pub fn lpush(&self, key: String, values: Vec<String>) -> Result<i64, String> {
        let mut data = self.data.lock().unwrap();

        match data.get_mut(&key) {
            Some(entry) => match &mut entry.value {
                StoreValue::ListVal(list) => {
                    for value in values {
                        list.push_front(value);
                    }

                    Ok(list.len() as i64)
                }

                _ => Err(WRONG_TYPE.to_string()),
            },
            None => {
                let mut list = VecDeque::new();

                for value in values {
                    list.push_front(value);
                }

                let len = list.len() as i64;

                data.insert(
                    key,
                    StoreEntry {
                        value: StoreValue::ListVal(list),
                        expires_at: None,
                    },
                );
                Ok(len)
            }
        }
    }

    //This function is intended to implement the RPUSH command, which adds one or more values to the tail of a list stored at a given key. If the key does not exist, it should create a new list. If the key exists but is not a list, it should return an error. The function should return the length of the list after the push operation.
    pub fn rpush(&self, key: String, values: Vec<String>) -> Result<i64, String> {
        let mut data = self.data.lock().unwrap();

        match data.get_mut(&key) {
            Some(entry) => match &mut entry.value {
                StoreValue::ListVal(list) => {
                    for value in values {
                        list.push_back(value);
                    }

                    Ok(list.len() as i64)
                }
                _ => Err(WRONG_TYPE.to_string()),
            },
            None => {
                let mut list = VecDeque::new();

                for value in values {
                    list.push_back(value);
                }

                let len = list.len() as i64;

                data.insert(
                    key,
                    StoreEntry {
                        value: StoreValue::ListVal(list),
                        expires_at: None,
                    },
                );
                Ok(len)
            }
        }
    }

    //This function is intended to implement the LPOP command, which removes and returns the first element of a list stored at a given key. If the key does not exist or is not a list, it should return an error. If the list is empty, it should return None.
    pub fn lpop(&self, key: &str) -> Result<Option<String>, String> {
        let mut data = self.data.lock().unwrap();

        match data.get_mut(key) {
            Some(entry) => match &mut entry.value {
                StoreValue::ListVal(list) => {
                    if list.is_empty() {
                        Ok(None)
                    } else {
                        Ok(list.pop_front())
                    }
                }
                _ => Err(WRONG_TYPE.to_string()),
            },
            None => Ok(None),
        }
    }

    //This function is intended to implement the RPOP command, which removes and returns the last element of a list stored at a given key. If the key does not exist or is not a list, it should return an error. If the list is empty, it should return None.
    pub fn rpop(&self, key: &str) -> Result<Option<String>, String> {
        let mut data = self.data.lock().unwrap();

        match data.get_mut(key) {
            Some(entry) => match &mut entry.value {
                StoreValue::ListVal(list) => {
                    if list.is_empty() {
                        Ok(None)
                    } else {
                        Ok(list.pop_back())
                    }
                }
                _ => Err(WRONG_TYPE.to_string()),
            },
            None => Ok(None),
        }
    }

    //This function is intended to implement the LRANGE command, which returns a range of elements from a list stored at a given key. The range is specified by the start and stop indices. If the key does not exist or is not a list, it should return an error. If the range is out of bounds, it should return an empty vector.
    pub fn lrange(&self, key: &str, start: i64, stop: i64) -> Result<Vec<String>, String> {
        let data = self.data.lock().unwrap();

        match data.get(key) {
            Some(entry) => {
                match &entry.value {
                    StoreValue::ListVal(list) => {
                        // Convert indices to usize for slicing
                        let start_idx = if start < 0 {
                            list.len() as i64 + start
                        } else {
                            start
                        } as usize;
                        let stop_idx = if stop < 0 {
                            list.len() as i64 + stop
                        } else {
                            stop
                        } as usize;

                        // Ensure indices are within bounds
                        let start_idx = start_idx.max(0);
                        let stop_idx = stop_idx.min(list.len() - 1);

                        if start_idx > stop_idx || start_idx >= list.len() {
                            return Ok(vec![]); // Return empty vector if range is invalid
                        }

                        // Return the specified range
                        Ok(list
                            .iter()
                            .skip(start_idx)
                            .take(stop_idx - start_idx + 1)
                            .cloned()
                            .collect())
                    }
                    _ => Err(WRONG_TYPE.to_string()),
                }
            }
            None => Ok(vec![]), // Return empty vector if key does not exist
        }
    }

    //This function runs in a loop, sleeping for 1 second between iterations. In each iteration, it locks the store's data and removes any entries that have expired. This ensures that expired keys are cleaned up regularly, preventing the store from growing indefinitely with stale data.
    pub async fn active_expiry_task(store: Arc<Store>) {
        loop {
            time::sleep(Duration::from_secs(1)).await;
            let mut data = store.data.lock().unwrap();
            let now = Instant::now();
            data.retain(|_, entry| match entry.expires_at {
                Some(expires_at) => expires_at > now,
                None => true,
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
