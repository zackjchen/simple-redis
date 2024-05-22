use crate::resp::frame::RespFrame;
use dashmap::{DashMap, DashSet};
use std::{ops::Deref, sync::Arc};

#[derive(Debug, Clone)]
pub struct Backend(Arc<BackendInner>);

#[derive(Debug, Default)]
pub struct BackendInner {
    map: DashMap<String, RespFrame>,
    hmap: DashMap<String, DashMap<String, RespFrame>>,
    set: DashMap<String, DashSet<RespFrame>>,
}

impl Deref for Backend {
    type Target = BackendInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self(Arc::new(BackendInner::default()))
    }
}

impl Backend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<RespFrame> {
        let value = self.map.get(key).map(|v| v.value().clone());
        value
    }

    pub fn set(&self, key: &str, value: RespFrame) -> Option<RespFrame> {
        self.map.insert(key.to_string(), value)
    }

    pub fn hget(&self, key: &str, field: &str) -> Option<RespFrame> {
        let value = self
            .hmap
            .get(key)
            .and_then(|m| m.get(field).map(|v| v.value().clone()));
        value
    }

    pub fn hget_all(&self, key: &str) -> Option<DashMap<String, RespFrame>> {
        self.hmap.get(key).map(|m| m.clone())
    }

    pub fn hset(&self, key: &str, field: &str, value: RespFrame) -> Option<RespFrame> {
        let map = self.hmap.entry(key.to_string()).or_default();
        map.insert(field.to_string(), value)
    }

    pub fn sadd(&self, key: &str, members: Vec<RespFrame>) {
        let mut set = self.set.entry(key.to_string()).or_default();
        set.extend(members);
    }

    pub fn sismembers(&self, key: &str, field: &RespFrame) -> bool {
        match self.set.get(key) {
            Some(set) => set.contains(field),
            None => false,
        }
    }
}
