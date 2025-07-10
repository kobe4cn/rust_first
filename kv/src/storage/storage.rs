use crate::{Kvpair, Value, error::KvError};

pub trait Storage:Send+Sync+'static {
    //获取一个key 的value
    fn get(&self, table: &str, key: &str) -> Result<Option<Value>, KvError>;
    //设置一个key 的value
    fn set(&self, table: &str, key: &str, value: Value) -> Result<Option<Value>, KvError>;
    //删除一个key
    fn delete(&self, table: &str, key: &str) -> Result<Option<Value>, KvError>;
    //判断一个key 是否存在
    fn contains(&self, table: &str, key: &str) -> Result<bool, KvError>;
    //获取一个表的所有key-value
    fn get_all(&self, table: &str) -> Result<Vec<Kvpair>, KvError>;
    //获取一个表的迭代器
    fn get_iter(&self, table: &str) -> Result<Box<dyn Iterator<Item = Kvpair>>, KvError>;
}

#[cfg(test)]
mod tests {

    use crate::MemTable;

    use super::*;

    #[test]

    fn memtable_basic_interface_should_work() {
        let storage = MemTable::new();
        test_basic_interface(storage);
    }

    fn test_basic_interface(storage: impl Storage) {
        let v = storage.set("t1", "k1", "v1".into());
        assert!(v.unwrap().is_none());

        let v1 = storage.set("t1", "k2", "v1".into());
        assert!(v1.unwrap().is_none());
        let v4 = storage.set("t1", "k4", "v1".into());
        assert!(v4.unwrap().is_none());

        let v1 = storage.set("t1", "k1", "v11".into());
        assert_eq!(v1, Ok(Some("v1".into())));

        let v = storage.get("t1", "k1");
        assert_eq!(v, Ok(Some("v11".into())));

        let v2 = storage.delete("t1", "k4");
        assert_eq!(v2, Ok(Some("v1".into())));

        assert_eq!(storage.get("t1", "k3"), Ok(None));

        assert!(storage.get("t2", "k1").unwrap().is_none());

        assert_eq!(storage.contains("t1", "k1"), Ok(true));

        assert_eq!(storage.contains("t2", "k1"), Ok(false));

        let vd = storage.delete("t1", "k2");
        assert_eq!(vd, Ok(Some("v1".into())));
    }
}
