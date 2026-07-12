use std::collections::HashMap;

use crate::resp::RESP;
use crate::set::{KeyExistence, KeyExpiry, SetArgs, parse_set_arguments};
use crate::storage_result::{StorageError, StorageResult};
use std::time::{Duration, SystemTime};

#[derive(Debug, PartialEq, Eq)]
pub enum StorageValue {
    String(String),
}
mod tests {
    use super::*;
    #[test]
    fn test_storage_date_eq() {
        let storage: Storage = Storage::new();
        assert!(storage.store.len() == 0);
        assert_eq!(storage.expiry.len(), 0);
        assert_eq!(storage.expiry, HashMap::new());
        assert!(storage.active_expire);
    }

    #[test]
    // Test that the function expire_keys removes
    // keys that have an expiry time in the past.
    fn test_expire_keys() {
        let mut storage: Storage = Storage::new();

        storage
            .set(String::from("akey"), String::from("avalue"), SetArgs::new())
            .unwrap();

        storage.expiry.insert(
            String::from("akey"),
            SystemTime::now() - Duration::from_secs(5),
        );

        storage.expire_keys();
        assert_eq!(storage.store.len(), 0);
    }

    #[test]
    // Test that the function expire_keys doesn't remove
    // keys that have an expiry time in the past
    // if active expiry is turned off.
    fn test_expire_keys_deactivated() {
        let mut storage: Storage = Storage::new();
        storage.active_expire = false;

        storage
            .set(String::from("akey"), String::from("avalue"), SetArgs::new())
            .unwrap();

        storage.expiry.insert(
            String::from("akey"),
            SystemTime::now() - Duration::from_secs(5),
        );

        storage.expire_keys();
        assert_eq!(storage.store.len(), 1);
    }

    #[test]
    // Test that get() performs passive expiration.
    fn test_get_passive_expire() {
        let mut storage: Storage = Storage::new();

        storage
            .set(String::from("akey"), String::from("avalue"), SetArgs::new())
            .unwrap();

        storage.expiry.insert(
            String::from("akey"),
            SystemTime::now() - Duration::from_secs(5),
        );

        let value = storage.get(String::from("akey")).unwrap();
        assert_eq!(value, None);
        assert_eq!(storage.store.len(), 0);
        assert_eq!(storage.expiry.len(), 0);
    }
}
pub struct StorageDate {
    pub value: StorageValue,
    pub creation_time: SystemTime,
    pub expiry: Option<Duration>,
}
pub struct Storage {
    store: HashMap<String, StorageDate>,
    expiry: HashMap<String, SystemTime>,
    active_expire: bool,
}
impl Storage {
    pub fn new() -> Self {
        let stroe: HashMap<String, StorageDate> = HashMap::new();
        Self {
            store: stroe,
            expiry: HashMap::new(),
            active_expire: true,
        }
    }

    pub fn expire_keys(&mut self) {
        if !self.active_expire {
            return;
        }
        let now = SystemTime::now();
        let expired_keys: Vec<String> = self
            .expiry
            .iter()
            .filter_map(|(key, value)| {
                if *value <= now {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect();
        for key in expired_keys.iter() {
            self.store.remove(key);
            self.expiry.remove(key);
        }
    }

    pub fn process_command(&mut self, command: Vec<String>) -> StorageResult<RESP> {
        match command[0].to_lowercase().as_str() {
            "ping" => self.command_ping(command),
            "echo" => self.command_echo(command),
            "info" => Ok(RESP::BulkString(String::from(
                "# Server\r\nredis_version:7.0.0\r\n",
            ))),
            "quit" => Ok(RESP::SimpleString(String::from("OK"))),
            "set" => self.command_set(command),
            "get" => self.command_get(command),
            _ => Err(StorageError::CommandNotAvaliable(command[0].clone())),
        }
    }
    fn command_ping(&mut self, command: Vec<String>) -> StorageResult<RESP> {
        Ok(RESP::SimpleString("PONG".to_string()))
    }
    fn command_echo(&mut self, command: Vec<String>) -> StorageResult<RESP> {
        Ok(RESP::SimpleString(command[1].clone()))
    }
    fn set(&mut self, key: String, value: String, args: SetArgs) -> StorageResult<Option<String>> {
        // NX/XX existence check based on current key presence (with passive expire).
        let key_exists = self.get(key.clone())?.is_some();
        match args.existence {
            Some(KeyExistence::NX) if key_exists => return Ok(None),
            Some(KeyExistence::XX) if !key_exists => return Ok(None),
            _ => {}
        }

        let mut data = StorageDate::from(value);
        if let Some(expiry) = args.expiry {
            let duration = match expiry {
                KeyExpiry::EX(v) => Duration::from_secs(v),
                KeyExpiry::PX(v) => Duration::from_millis(v),
            };
            data.add_expiry(duration);
            self.expiry
                .insert(key.clone(), SystemTime::now() + duration);
        } else {
            // Clear any previous expiry if none is specified.
            self.expiry.remove(&key);
        }
        self.store.insert(key, data);
        Ok(Some("OK".to_string()))
    }
    fn get(&mut self, key: String) -> StorageResult<Option<String>> {
        // Passive expiration: check and remove expired keys on access.
        if let Some(&expiry_time) = self.expiry.get(&key) {
            if expiry_time <= SystemTime::now() {
                self.store.remove(&key);
                self.expiry.remove(&key);
                return Ok(None);
            }
        }

        match self.store.get(&key) {
            Some(StorageDate {
                value: StorageValue::String(v),
                ..
            }) => Ok(Some(v.clone())),
            None => Ok(None),
        }
    }
    fn command_set(&mut self, command: Vec<String>) -> StorageResult<RESP> {
        if command.len() < 3 {
            return Err(StorageError::IncorrectRequest);
        }
        let set_args = parse_set_arguments(&command[3..].to_vec())?;
        let result = self.set(command[1].clone(), command[2].clone(), set_args)?;
        match result {
            Some(_) => Ok(RESP::SimpleString("OK".to_string())),
            None => Ok(RESP::Null),
        }
    }
    fn command_get(&mut self, command: Vec<String>) -> StorageResult<RESP> {
        if command.len() != 2 {
            return Err(StorageError::IncorrectRequest);
        }
        let output = self.get(command[1].clone());
        match output {
            Ok(Some(v)) => Ok(RESP::BulkString(v)),
            Ok(None) => Ok(RESP::Null),
            Err(_) => Err(StorageError::CommandNotAvaliable(command.join(" "))),
        }
    }
}

impl StorageDate {
    pub fn add_expiry(&mut self, expiry: Duration) {
        self.expiry = Some(expiry);
    }
}

impl From<String> for StorageDate {
    fn from(s: String) -> Self {
        let value = StorageValue::String(s);
        let creation_time = SystemTime::now();
        let expiry = None;
        Self {
            value,
            creation_time,
            expiry,
        }
    }
}

impl PartialEq for StorageDate {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.expiry == other.expiry
    }
}
