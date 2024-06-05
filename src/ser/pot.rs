use std::{collections::HashMap, str::from_utf8};

use super::SerializeMethod;
use crate::{DbListMap, DbMap};
use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Serialize};

use pot::{from_slice, to_vec};

pub(crate) struct PotSer;

impl PotSer {
    pub(crate) const fn new() -> Self {
        PotSer
    }
}

impl SerializeMethod for PotSer {
    fn serialize_data<T: Serialize>(&self, data: &T) -> Result<Vec<u8>> {
        Ok(to_vec(data)?)
    }
    fn serialize_db(&self, db_map: &DbMap, db_list_map: &DbListMap) -> Result<Vec<u8>> {
        let mut map = HashMap::new();
        for (k, v) in db_map.iter() {
            map.insert(k.as_str(), from_utf8(v)?);
        }
        let mut list_map = HashMap::new();
        for (k, v) in db_list_map.iter() {
            let list = v
                .iter()
                .map(|x| from_utf8(x).unwrap_or(""))
                .collect::<Vec<_>>();
            list_map.insert(k.as_str(), list);
        }
        Ok(to_vec(&(map, list_map))?)
    }
    fn deserialize_data<T: DeserializeOwned>(&self, data: &[u8]) -> Option<T> {
        from_slice(data).ok()
    }
    fn deserialized_db(&self, ser_db: &[u8]) -> Result<(DbMap, DbListMap)> {
        match from_slice::<(HashMap<String, String>, HashMap<String, Vec<String>>)>(ser_db) {
            Ok((map, list_map)) => {
                let mut db_map = HashMap::new();
                for (k, v) in map.iter() {
                    db_map.insert(k.to_string(), v.as_bytes().to_vec());
                }
                let mut db_list_map = HashMap::new();
                for (k, v) in list_map.iter() {
                    let list = v.iter().map(|x| x.as_bytes().to_vec()).collect::<Vec<_>>();
                    db_list_map.insert(k.to_string(), list);
                }
                Ok((db_map, db_list_map))
            }
            Err(e) => Err(anyhow!(e)),
        }
    }
}
