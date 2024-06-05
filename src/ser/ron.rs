use super::SerializeMethod;
use crate::{DbListMap, DbMap};
use anyhow::{anyhow, Result};
use ron::{de::from_bytes, ser::to_string};
use serde::{de::DeserializeOwned, Serialize};

pub struct RonSer;

impl RonSer {
    pub fn new() -> Self {
        RonSer
    }
}

impl SerializeMethod for RonSer {
    fn serialize_data<T: Serialize>(&self, data: &T) -> Result<Vec<u8>> {
        Ok(to_string(data)?.into_bytes())
    }
    fn serialize_db(&self, db_map: &DbMap, db_list_map: &DbListMap) -> Result<Vec<u8>> {
        self.serialize_data(&(db_map, db_list_map))
    }
    fn deserialize_data<T: DeserializeOwned>(&self, data: &[u8]) -> Option<T> {
        from_bytes(data).ok()
    }
    fn deserialized_db(&self, ser_db: &[u8]) -> Result<(DbMap, DbListMap)> {
        match self.deserialize_data(ser_db) {
            Some((db_map, db_list_map)) => Ok((db_map, db_list_map)),
            None => Err(anyhow!("Failed to deserialize db")),
        }
    }
}
