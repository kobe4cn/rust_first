use std::sync::Arc;

use tracing::{debug, info};

use crate::{
    CommandRequest, CommandResponse, MemTable, command::commandservice::dispatch,
    storage::storage::Storage,
};

pub struct Service<S = MemTable> {
    inner: Arc<ServiceInner<S>>,
}

impl<S: Storage> Service<S> {
    pub fn exec(&self, cmd: CommandRequest) -> CommandResponse {
        debug!("Got request: {:?}", cmd);
        self.inner.on_received.notify(&cmd);
        let mut res = dispatch(cmd, &self.inner.store);
        debug!("Exec result: {:?}", res);
        self.inner.on_executed.notify(&res);
        self.inner.on_berfore_send.notify(&mut res);
        if !self.inner.on_after_send.is_empty() {
            info!("modify response: {:?}", res);
        }
        res
    }
}

impl<S: Storage> Clone for Service<S> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

pub struct ServiceInner<S> {
    store: S,
    on_received: Vec<fn(&CommandRequest)>,
    on_executed: Vec<fn(&CommandResponse)>,
    on_berfore_send: Vec<fn(&mut CommandResponse)>,
    on_after_send: Vec<fn()>,
}

impl<S: Storage> ServiceInner<S> {
    pub fn new(store: S) -> Self {
        Self {
            store,
            on_received: Vec::new(),
            on_executed: Vec::new(),
            on_berfore_send: Vec::new(),
            on_after_send: Vec::new(),
        }
    }

    pub fn fn_received(mut self, f: fn(&CommandRequest)) -> Self {
        self.on_received.push(f);
        self
    }

    pub fn fn_executed(mut self, f: fn(&CommandResponse)) -> Self {
        self.on_executed.push(f);
        self
    }

    pub fn fn_berfore_send(mut self, f: fn(&mut CommandResponse)) -> Self {
        self.on_berfore_send.push(f);
        self
    }

    pub fn fn_after_send(mut self, f: fn()) -> Self {
        self.on_after_send.push(f);
        self
    }
}

impl<S: Storage> From<ServiceInner<S>> for Service<S> {
    fn from(inner: ServiceInner<S>) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }
}

pub trait Nofiy<Arg> {
    fn notify(&self, arg: &Arg);
}

pub trait NotifyMut<Arg> {
    fn notify(&self, arg: &mut Arg);
}

impl<Arg> Nofiy<Arg> for Vec<fn(&Arg)> {
    fn notify(&self, arg: &Arg) {
        for f in self {
            f(&arg);
        }
    }
}

impl<Arg> NotifyMut<Arg> for Vec<fn(&mut Arg)> {
    fn notify(&self, arg: &mut Arg) {
        for f in self {
            f(arg);
        }
    }
}

#[cfg(test)]
mod tests {
    use http::StatusCode;
    use tracing::info;

    use crate::{MemTable, Value};

    use super::*;

    #[test]
    fn event_registration_should_work() {
        fn b(cmd: &CommandRequest) {
            info!("on_received: {:?}", cmd);
        }
        fn c(cmd: &CommandResponse) {
            info!("{:?}", cmd);
        }
        fn d(cmd: &mut CommandResponse) {
            cmd.status = StatusCode::CREATED.as_u16() as _;
        }
        fn e() {
            info!("on_after_send");
        }

        let service: Service = ServiceInner::new(MemTable::default())
            .fn_received(b)
            .fn_executed(c)
            .fn_berfore_send(d)
            .fn_after_send(e)
            .into();

        let res = service.exec(CommandRequest::new_hset("t1", "k1", "v1".into()));
        assert_eq!(res.status, StatusCode::CREATED.as_u16() as _);
        assert_eq!(res.message, "success");
        assert_eq!(res.values, vec![Value::default()]);
    }
}
