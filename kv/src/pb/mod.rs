use crate::{
    CommandRequest, CommandResponse, Hget, Hgetall, Hset, Kvpair, Value,
    command_request::RequestData, error::KvError, value,
};

pub mod abi;

impl CommandRequest {
    pub fn new_hset(table: &str, key: &str, value: Value) -> Self {
        Self {
            request_data: Some(RequestData::Hset(Hset {
                table: table.into(),
                pair: Some(Kvpair::new(key, value)),
            })),
        }
    }
    pub fn new_hget(table: &str, key: &str) -> Self {
        Self {
            request_data: Some(RequestData::Hget(Hget {
                table: table.into(),
                key: key.into(),
            })),
        }
    }
    pub fn new_hgetall(table: &str) -> Self {
        Self {
            request_data: Some(RequestData::Hgetall(Hgetall {
                table: table.into(),
            })),
        }
    }
}

impl Kvpair {
    pub fn new(key: &str, value: Value) -> Self {
        Self {
            key: key.into(),
            value: Some(value),
        }
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self {
            value: Some(value::Value::StringValue(value)),
        }
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self {
            value: Some(value::Value::StringValue(value.into())),
        }
    }
}

impl From<Value> for CommandResponse {
    fn from(value: Value) -> Self {
        Self {
            status: 200,
            message: "success".to_string(),
            values: vec![value.into()],
            pairs: vec![],
        }
    }
}

impl From<KvError> for CommandResponse {
    fn from(error: KvError) -> Self {
        match error {
            KvError::KeyNotFound => Self {
                status: 404,
                message: "not found".to_string(),
                values: vec![],
                pairs: vec![],
            },
            _ => Self {
                status: 500,
                message: error.to_string(),
                values: vec![],
                pairs: vec![],
            },
        }
    }
}

impl From<Vec<Kvpair>> for CommandResponse {
    fn from(value: Vec<Kvpair>) -> Self {
        Self {
            status: 200,
            message: "success".to_string(),
            values: vec![],
            pairs: value,
        }
    }
}
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self {
            value: Some(value::Value::Int64Value(value)),
        }
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self {
            value: Some(value::Value::DoubleValue(value)),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self {
            value: Some(value::Value::BoolValue(value)),
        }
    }
}
