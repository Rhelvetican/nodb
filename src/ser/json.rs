use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_str, to_string};
use std::{collections::HashMap, str::from_utf8};

use super::{DbListMap, DbMap, SerializeMethod};

pub struct JsonSer {}

impl JsonSer {
    pub fn new() -> Self {
        JsonSer {}
    }
}

impl SerializeMethod for JsonSer {
    fn serialize_data<T: Serialize>(&self, data: &T) -> Result<Vec<u8>> {
        let val = to_string(data)?;
        Ok(val.as_bytes().to_vec())
    }
    fn deserialize_data<T: DeserializeOwned>(&self, data: &[u8]) -> Option<T> {
        from_str(match from_utf8(data).ok() {
            Some(v) => v,
            None => return None,
        })
        .ok()
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
        Ok(to_string(&(map, list_map))?.into_bytes())
    }
    fn deserialized_db(&self, ser_db: &[u8]) -> Result<(DbMap, DbListMap)> {
        match from_str::<(HashMap<String, String>, HashMap<String, Vec<String>>)>(from_utf8(
            ser_db,
        )?) {
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
