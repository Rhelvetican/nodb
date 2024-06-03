use std::{
    fs::{read, rename, write},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    ext::NoDbExt,
    iter::{NoDbIter, NoDbListIter},
    ser::{SerializationMethod, SerializeMethod, Serializer},
    DbListMap, DbMap,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DumpPolicy {
    Never,
    Auto,
    #[default]
    OnCall,
    Periodic(Duration),
}

pub struct NoDb {
    pub map: DbMap,
    pub list_map: DbListMap,
    pub ser: Serializer,
    pub path: PathBuf,
    pub policy: DumpPolicy,
    pub last_dump: Instant,
}

impl NoDb {
    pub fn new<P: AsRef<Path>>(
        db_path: P,
        policy: DumpPolicy,
        ser_method: SerializationMethod,
    ) -> Self {
        let path = db_path.as_ref().to_path_buf();

        NoDb {
            map: DbMap::new(),
            list_map: DbListMap::new(),
            ser: Serializer::from(ser_method),
            path,
            policy,
            last_dump: Instant::now(),
        }
    }
    pub fn load<P: AsRef<Path>>(
        db_path: P,
        policy: DumpPolicy,
        ser_method: SerializationMethod,
    ) -> Result<Self> {
        let content = read(&db_path)?;
        let ser = Serializer::from(ser_method);
        let (map, list_map) = ser.deserialized_db(&content)?;
        let path_buf = db_path.as_ref().to_path_buf();

        Ok(NoDb {
            map,
            list_map,
            ser,
            path: path_buf,
            policy,
            last_dump: Instant::now(),
        })
    }
    pub fn dump(&mut self) -> Result<()> {
        if let DumpPolicy::Never = self.policy {
            return Ok(());
        }
        let data = self.ser.serialize_db(&self.map, &self.list_map)?;
        let tmp = format!(
            "{}.tmp.{}",
            self.path.to_str().unwrap_or("db"),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );

        write(&tmp, data)?;
        rename(&tmp, &self.path)?;
        if let DumpPolicy::Periodic(_) = self.policy {
            self.last_dump = Instant::now();
        }
        Ok(())
    }
    fn dumpdb(&mut self) -> Result<()> {
        match self.policy {
            DumpPolicy::Auto => self.dump(),
            DumpPolicy::Periodic(dur) => {
                let now = Instant::now();
                if now.duration_since(self.last_dump) >= dur {
                    self.dump()
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
    pub fn set<K: AsRef<str>, V: Serialize>(&mut self, key: K, value: V) -> Result<()> {
        let key = key.as_ref();
        if self.list_map.contains_key(key) {
            self.list_map.remove(key);
        }
        let data = self.ser.serialize_data(&value)?;
        let orig_val = self.map.insert(key.to_string(), data);
        match self.dumpdb() {
            Ok(_) => Ok(()),
            Err(err) => {
                match orig_val {
                    Some(val) => self.map.insert(String::from(key), val),
                    None => self.map.remove(key),
                };
                Err(err)
            }
        }
    }
    pub fn get<K: AsRef<str>, V: DeserializeOwned>(&self, key: K) -> Option<V> {
        let key = key.as_ref();
        let res = self.map.get(key);
        if let Some(v) = res {
            self.ser.deserialize_data(v)
        } else {
            None
        }
    }
    pub fn exists<K: AsRef<str>>(&self, key: K) -> bool {
        self.map.contains_key(key.as_ref()) || self.list_map.contains_key(key.as_ref())
    }
    pub fn get_all(&self) -> Vec<String> {
        [
            self.map.keys().cloned().collect::<Vec<String>>(),
            self.list_map.keys().cloned().collect::<Vec<String>>(),
        ]
        .concat()
    }
    pub fn total_keys(&self) -> usize {
        self.map.iter().len() + self.list_map.iter().len()
    }
    pub fn rem<K: AsRef<str>>(&mut self, key: K) -> Result<bool> {
        let key = key.as_ref();
        let rm_map = match self.map.remove(key) {
            None => None,
            Some(val) => match self.dumpdb() {
                Ok(_) => Some(val),
                Err(err) => {
                    self.map.insert(String::from(key), val);
                    return Err(err);
                }
            },
        };
        let rm_list_map = match self.list_map.remove(key) {
            None => None,
            Some(val) => match self.dumpdb() {
                Ok(_) => Some(val),
                Err(err) => {
                    self.list_map.insert(String::from(key), val);
                    return Err(err);
                }
            },
        };
        Ok(rm_map.is_some() || rm_list_map.is_some())
    }
    pub fn ladd<K: AsRef<str>, V: Serialize>(&mut self, name: K, value: &V) -> Option<NoDbExt> {
        self.lextend(name, &[value])
    }
    pub fn lextend<'a, N: AsRef<str>, V, I>(&mut self, name: N, seq: I) -> Option<NoDbExt>
    where
        V: 'a + Serialize,
        I: IntoIterator<Item = &'a V>,
    {
        let ser = &self.ser;
        match self.list_map.get_mut(name.as_ref()) {
            Some(list) => {
                let orig_len = list.len();
                let serialized = seq
                    .into_iter()
                    .map(|v| ser.serialize_data(v).unwrap())
                    .collect::<Vec<_>>();
                list.extend(serialized);
                match self.dumpdb() {
                    Ok(_) => (),
                    Err(_) => {
                        let same_list = self.list_map.get_mut(name.as_ref()).unwrap();
                        same_list.truncate(orig_len);
                        return None;
                    }
                }
                Some(NoDbExt {
                    db: self,
                    list_name: name.as_ref().to_string(),
                })
            }
            None => None,
        }
    }
    pub fn lget<V: DeserializeOwned, N: AsRef<str>>(&self, name: N, pos: usize) -> Option<V> {
        match self.list_map.get(name.as_ref()) {
            Some(list) => match list.get(pos) {
                Some(val) => self.ser.deserialize_data::<V>(val),
                None => None,
            },
            None => None,
        }
    }
    pub fn llen<N: AsRef<str>>(&self, name: N) -> usize {
        match self.list_map.get(name.as_ref()) {
            Some(list) => list.len(),
            None => 0,
        }
    }
    pub fn lrem_list<N: AsRef<str>>(&mut self, name: N) -> Result<usize> {
        let res = self.llen(&name);
        let name = name.as_ref();
        match self.list_map.remove(name) {
            Some(list) => match self.dumpdb() {
                Ok(_) => Ok(res),
                Err(err) => {
                    self.list_map.insert(String::from(name), list);
                    Err(err)
                }
            },
            None => Ok(res),
        }
    }
    pub fn lpop<V: DeserializeOwned, N: AsRef<str>>(&mut self, name: N, pos: usize) -> Option<V> {
        let name = name.as_ref();
        match self.list_map.get_mut(name) {
            Some(list) => {
                if pos < list.len() {
                    let res = list.remove(pos);
                    match self.dumpdb() {
                        Ok(_) => self.ser.deserialize_data::<V>(&res),
                        Err(_) => {
                            let same_list = self.list_map.get_mut(name).unwrap();
                            same_list.insert(pos, res);
                            None
                        }
                    }
                } else {
                    None
                }
            }

            None => None,
        }
    }
    pub fn lrem_value<V: Serialize, N: AsRef<str>>(&mut self, name: N, value: &V) -> Result<bool> {
        let name = name.as_ref();
        match self.list_map.get_mut(name) {
            Some(list) => {
                let serialized_value = match self.ser.serialize_data(&value) {
                    Ok(val) => val,
                    Err(err) => {
                        return Err(anyhow!(
                            "Error serializing value: {}",
                            err.to_string().replace('\n', "")
                        ))
                    }
                };

                match list.iter().position(|x| *x == serialized_value) {
                    Some(pos) => {
                        list.remove(pos);
                        match self.dumpdb() {
                            Ok(_) => Ok(true),
                            Err(err) => {
                                let same_list = self.list_map.get_mut(name).unwrap();
                                same_list.insert(pos, serialized_value);
                                Err(err)
                            }
                        }
                    }

                    None => Ok(false),
                }
            }

            None => Ok(false),
        }
    }
    pub fn iter(&self) -> NoDbIter {
        NoDbIter {
            map_iter: self.map.iter(),
            ser: &self.ser,
        }
    }
    pub fn liter<N: AsRef<str>>(&self, name: N) -> NoDbListIter {
        let name = name.as_ref();
        match self.list_map.get(name) {
            Some(list) => NoDbListIter {
                list_iter: list.iter(),
                ser: &self.ser,
            },
            None => NoDbListIter {
                list_iter: [].iter(),
                ser: &self.ser,
            },
        }
    }
}

impl Drop for NoDb {
    fn drop(&mut self) {
        if !matches!(self.policy, DumpPolicy::Never | DumpPolicy::OnCall) {
            let _ = self.dump();
        }
    }
}
