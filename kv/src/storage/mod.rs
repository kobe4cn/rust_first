use dashmap::{DashMap, mapref::one::Ref};

use crate::{Kvpair, Value, error::KvError, storage::storage::Storage};

pub mod storage;

#[derive(Debug, Clone, Default)]
pub struct MemTable {
    table: DashMap<String, DashMap<String, Value>>,
}

impl MemTable {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_or_create_table(&self, table: &str) -> Ref<String, DashMap<String, Value>> {
        match self.table.get(table) {
            Some(table) => table,
            None => {
                let entry = self.table.entry(table.into()).or_default();
                entry.downgrade()
            }
        }
    }
}
impl Storage for MemTable {
    fn get(&self, table: &str, key: &str) -> Result<Option<Value>, KvError> {
        let table = self.get_or_create_table(table);
        Ok(table.get(key).map(|v| v.clone()))
    }
    fn set(&self, table: &str, key: &str, value: Value) -> Result<Option<Value>, KvError> {
        let table = self.get_or_create_table(table);
        let v = table.insert(key.into(), value);
        Ok(v)
    }
    fn delete(&self, table: &str, key: &str) -> Result<Option<Value>, KvError> {
        let table = self.get_or_create_table(table);
        Ok(table.remove(key).map(|(_, v)| v))
    }
    fn contains(&self, table: &str, key: &str) -> Result<bool, KvError> {
        let table = self.get_or_create_table(table);
        Ok(table.contains_key(key))
    }
    fn get_all(&self, table: &str) -> Result<Vec<Kvpair>, KvError> {
        let table = self.get_or_create_table(table);
        Ok(table
            .iter()
            .map(|kv| Kvpair::new(kv.key(), kv.value().clone()))
            .collect())
    }
    fn get_iter(&self, table: &str) -> Result<Box<dyn Iterator<Item = Kvpair>>, KvError> {
        let table = self.get_or_create_table(table);
        let pairs: Vec<Kvpair> = table
            .iter()
            .map(|kv| Kvpair::new(kv.key(), kv.value().clone()))
            .collect();
        Ok(Box::new(pairs.into_iter()))
    }
}
