use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Serialize};
use serde_cbor::{from_slice, to_vec};

use super::{DbListMap, DbMap, SerializeMethod};

pub struct CborSer;

impl CborSer {
    pub fn new() -> Self {
        CborSer
    }
}

impl SerializeMethod for CborSer {
    fn serialize_data<T: Serialize>(&self, data: &T) -> Result<Vec<u8>> {
        Ok(to_vec(data)?)
    }
    fn serialize_db(&self, db_map: &DbMap, db_list_map: &DbListMap) -> Result<Vec<u8>> {
        self.serialize_data(&(db_map, db_list_map))
    }
    fn deserialize_data<T: DeserializeOwned>(&self, data: &[u8]) -> Option<T> {
        from_slice(data).ok()
    }
    fn deserialized_db(&self, ser_db: &[u8]) -> Result<(DbMap, DbListMap)> {
        match self.deserialize_data(ser_db) {
            Some((db_map, db_list_map)) => Ok((db_map, db_list_map)),
            None => Err(anyhow!("Failed to deserialize db")),
        }
    }
}
