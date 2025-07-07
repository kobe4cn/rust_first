use crate::{
    CommandRequest, CommandResponse, Hget, Hgetall, Hset, Value, command_request::RequestData,
    error::KvError, storage::storage::Storage,
};

pub trait CommandService {
    fn exec(&self, storage: &impl Storage) -> CommandResponse;
}
#[allow(unused)]
pub fn dispatch(cmd: CommandRequest, storage: &impl Storage) -> CommandResponse {
    match cmd.request_data {
        Some(RequestData::Hget(params)) => params.exec(storage),
        Some(RequestData::Hgetall(params)) => params.exec(storage),
        Some(RequestData::Hset(params)) => params.exec(storage),
        // Some(RequestData::Hmget(params)) => params.exec(storage),
        // Some(RequestData::Hmset(params)) => params.exec(storage),
        // Some(RequestData::Hdel(params)) => params.exec(storage),
        // Some(RequestData::Hmdel(params)) => params.exec(storage),
        // Some(RequestData::Hexists(params)) => params.exec(storage),
        // Some(RequestData::Hmexists(params)) => params.exec(storage),
        _ => KvError::Internal("Not implemented".into()).into(),
    }
}

impl CommandService for Hget {
    fn exec(&self, storage: &impl Storage) -> CommandResponse {
        match storage.get(&self.table, &self.key) {
            Ok(Some(value)) => value.into(),
            Ok(None) => KvError::KeyNotFound.into(),
            Err(e) => e.into(),
        }
    }
}

impl CommandService for Hset {
    fn exec(&self, storage: &impl Storage) -> CommandResponse {
        if let Some(pair) = self.pair.as_ref() {
            if pair.value.is_none() {
                return KvError::Internal("value is required".into()).into();
            } else {
                match storage.set(&self.table, &pair.key, pair.value.clone().unwrap()) {
                    Ok(Some(value)) => value.into(),
                    Ok(None) => Value::default().into(),
                    Err(e) => e.into(),
                }
            }
        } else {
            KvError::Internal("pair is required".into()).into()
        }
    }
}

impl CommandService for Hgetall {
    fn exec(&self, storage: &impl Storage) -> CommandResponse {
        match storage.get_all(&self.table) {
            Ok(pairs) => pairs.into(),
            Err(e) => e.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Kvpair, MemTable, Value};

    use super::*;

    #[test]
    fn hset_should_work() {
        let table = MemTable::new();
        let cmd = CommandRequest::new_hset("t1", "k1", "v1".into());
        let _cmd2 = CommandRequest::new_hset("t1", "k2", "v2".into());

        let resp = dispatch(cmd.clone(), &table);

        assert_res_ok(resp, &[Value::default()], &[]);

        let resp = dispatch(cmd, &table);
        assert_res_ok(resp, &["v1".into()], &[]);
    }

    #[test]
    fn hget_should_work() {
        let table = MemTable::new();
        let cmd = CommandRequest::new_hset("t1", "k1", "v1".into());

        dispatch(cmd.clone(), &table);

        let cmd = CommandRequest::new_hget("t1", "k1");
        let resp = dispatch(cmd, &table);
        assert_res_ok(resp, &["v1".into()], &[]);

        let cmd = CommandRequest::new_hget("t1", "k2");
        let resp = dispatch(cmd, &table);
        assert_res_error(resp, 404, "not found");
    }

    #[test]
    fn hgetall_should_work() {
        let table = MemTable::new();
        let cmds = vec![
            CommandRequest::new_hset("t1", "k1", "v1".into()),
            CommandRequest::new_hset("t1", "k2", "v2".into()),
            CommandRequest::new_hset("t1", "k3", "v3".into()),
        ];
        for cmd in cmds {
            dispatch(cmd.clone(), &table);
        }
        let cmd = CommandRequest::new_hgetall("t1");
        let resp = dispatch(cmd, &table);
        assert_res_ok(
            resp,
            &[],
            &[
                Kvpair::new("k1", "v1".into()),
                Kvpair::new("k2", "v2".into()),
                Kvpair::new("k3", "v3".into()),
            ],
        );
    }

    fn assert_res_ok(mut res: CommandResponse, values: &[Value], pairs: &[Kvpair]) {
        res.pairs.sort_by(|a, b| a.key.cmp(&b.key));
        assert_eq!(res.status, 200);
        assert_eq!(res.message, "success");
        assert_eq!(res.values, values);
        assert_eq!(res.pairs, pairs);
    }
    #[allow(unused)]
    fn assert_res_error(res: CommandResponse, code: u32, msg: &str) {
        assert_eq!(res.status, code);
        assert_eq!(res.message, msg);
        assert_eq!(res.values, &[]);
        assert_eq!(res.pairs, &[]);
    }
}
