use std::sync::Arc;

use tracing::debug;

use crate::{
    CommandRequest, CommandResponse, command::commandservice::dispatch, storage::storage::Storage,
};

pub struct Service {
    pub store: Arc<dyn Storage>,
}

impl Service {
    pub fn new<Store: Storage>(store: Store) -> Self {
        Self {
            store: Arc::new(store),
        }
    }

    pub fn exec(&self, cmd: CommandRequest) -> CommandResponse {
        debug!("Got request: {:?}", cmd);
        let res = dispatch(cmd, self.store.as_ref());
        debug!("Exec result: {:?}", res);
        res
    }
}

impl Clone for Service {
    fn clone(&self) -> Self {
        Self {
            store: Arc::clone(&self.store),
        }
    }
}
