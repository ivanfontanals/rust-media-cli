use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;

#[async_trait(?Send)]
pub trait Store<K, V>: Send
where
    K: Serialize,
    V: Serialize,
{
    fn get(&self, key: K) -> Option<&V>;
    fn put(&mut self, key: K, value: V) -> Result<()>;
}
