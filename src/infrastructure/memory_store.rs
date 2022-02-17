use crate::domain::ports::outgoing::Store;
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct MemoryStore<K, V>
where
    K: Serialize,
    V: Serialize,
{
    map: HashMap<K, V>,
}

impl<K, V> MemoryStore<K, V>
where
    K: Serialize + Send,
    V: Serialize + Send,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::<K, V>::new(),
        }
    }
}

impl<K, V> Default for MemoryStore<K, V>
where
    K: Serialize + Send,
    V: Serialize + Send,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Store<K, V> for MemoryStore<K, V>
where
    K: Serialize + Send + Eq + Hash,
    V: Serialize + Send,
{
    fn put(&mut self, key: K, value: V) -> Result<()> {
        self.map.insert(key, value);
        Ok(())
    }
    fn get(&self, key: K) -> Option<&V> {
        self.map.get(&key)
    }
}
