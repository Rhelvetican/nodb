use anyhow::{anyhow, Result};
use bincode::{deserialize, serialize};
use serde::{de::DeserializeOwned, Serialize};

use super::{DbListMap, DbMap, SerializeMethod};

pub(crate) struct BinSer;

impl BinSer {
    pub(crate) const fn new() -> Self {
        BinSer
    }
}

impl SerializeMethod for BinSer {
    fn serialize_data<T: Serialize>(&self, data: &T) -> Result<Vec<u8>> {
        Ok(serialize(data)?)
    }
    fn deserialize_data<T: DeserializeOwned>(&self, data: &[u8]) -> Option<T> {
        deserialize(data).ok()
    }
    fn serialize_db(&self, db_map: &DbMap, db_list_map: &DbListMap) -> Result<Vec<u8>> {
        self.serialize_data(&(db_map, db_list_map))
    }
    fn deserialized_db(&self, ser_db: &[u8]) -> Result<(DbMap, DbListMap)> {
        match self.deserialize_data(ser_db) {
            Some((db_map, db_list_map)) => Ok((db_map, db_list_map)),
            None => Err(anyhow!("Failed to deserialize db")),
        }
    }
}
