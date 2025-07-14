use std::path::Path;

use sled::{Db, IVec};

use crate::{Kvpair, StorageIter, Value, error::KvError, storage::storage::Storage};

pub struct SledDb(Db);

impl SledDb {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let db = sled::open(path).unwrap();
        Self(db)
    }

    fn get_full_key(table: &str, key: &str) -> String {
        format!("{}:{}", table, key)
    }

    fn get_table_perfix(table: &str) -> String {
        format!("{}:", table)
    }
}

fn flip<T, E>(x: Option<Result<T, E>>) -> Result<Option<T>, E> {
    x.map_or(Ok(None), |x| x.map(Some))
}

impl Storage for SledDb {
    fn get(&self, table: &str, key: &str) -> Result<Option<Value>, KvError> {
        let full_key = SledDb::get_full_key(table, key);
        let value = self
            .0
            .get(full_key.as_bytes())?
            .map(|v| Value::try_from(v.as_ref()));
        flip(value)
    }
    fn set(&self, table: &str, key: &str, value: Value) -> Result<Option<Value>, KvError> {
        let full_key = SledDb::get_full_key(table, key);
        let value: Vec<u8> = value.try_into()?;
        let result = self
            .0
            .insert(full_key, value)?
            .map(|v| Value::try_from(v.as_ref()));
        flip(result)
    }
    fn delete(&self, table: &str, key: &str) -> Result<Option<Value>, KvError> {
        let full_key = SledDb::get_full_key(table, key);
        let result = self
            .0
            .remove(full_key.as_bytes())?
            .map(|v| Value::try_from(v.as_ref()));
        flip(result)
    }
    fn contains(&self, table: &str, key: &str) -> Result<bool, KvError> {
        let full_key = SledDb::get_full_key(table, key);
        let result = self.0.contains_key(full_key.as_bytes())?;
        Ok(result)
    }

    fn get_all(&self, table: &str) -> Result<Vec<Kvpair>, KvError> {
        let prefix = SledDb::get_table_perfix(table);
        let result = self
            .0
            .scan_prefix(prefix.as_bytes())
            .map(|v| v.into())
            .collect();
        Ok(result)
    }

    fn get_iter(&self, table: &str) -> Result<Box<dyn Iterator<Item = Kvpair>>, KvError> {
        let prefix = SledDb::get_table_perfix(table);
        let result = StorageIter::new(self.0.scan_prefix(prefix.as_bytes()));
        Ok(Box::new(result))
    }
}

impl From<Result<(IVec, IVec), sled::Error>> for Kvpair {
    fn from(value: Result<(IVec, IVec), sled::Error>) -> Self {
        match value {
            Ok((k, v)) => match v.as_ref().try_into() {
                Ok(v) => Kvpair::new(ivec_to_str(k.as_ref()), v),
                Err(_) => Kvpair::default(),
            },
            _ => Kvpair::default(),
        }
    }
}

fn ivec_to_str(ivec: &[u8]) -> &str {
    let s = str::from_utf8(ivec).unwrap();
    let mut iter = s.split(":");
    iter.next();
    iter.next().unwrap()
}
