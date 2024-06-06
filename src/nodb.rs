use std::{
    fs::{read, rename, write, DirBuilder},
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    crypto::B64,
    ext::NoDbExt,
    iter::{NoDbIter, NoDbListIter},
    ser::{SerializationMethod, SerializeMethod, Serializer},
    DbListMap, DbMap,
};

const B64: B64 = B64::new();

/// An enum that determines the policy of dumping NoDb changes into the file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DumpPolicy {
    /// Never dump the changes into the file
    Never,
    /// Every change will be dumped immediately and automatically to the file
    Auto,
    #[default]
    /// Data won't be dumped unless the developer calls [NoDb::dump()](struct.NoDb.html#method.dump) proactively to dump the data
    OnCall,
    /// Changes will be dumped to the file periodically, no sooner than the Duration provided by the developer.
    /// The way this mechanism works is as follows: each time there is a DB change the last DB dump time is checked.
    /// If the time that has passed since the last dump is higher than Duration, changes will be dumped,
    /// otherwise changes will not be dumped.
    Periodic(Duration),
}

/// A struct that represents a NoDb object.
pub struct NoDb {
    pub map: DbMap,
    pub list_map: DbListMap,
    ser: Serializer,
    pub path: PathBuf,
    pub policy: DumpPolicy,
    pub last_dump: Instant,
}

impl NoDb {
    /// Constructs a new `NoDb` instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nodb::{NoDb, DumpPolicy, SerializationMethod};
    ///
    /// let mut db = NoDb::new("example.db", DumpPolicy::AutoDump, SerializationMethod::Json);
    /// ```

    pub fn new<P: AsRef<Path>>(
        db_path: P,
        policy: DumpPolicy,
        ser_method: SerializationMethod,
    ) -> Self {
        let path = db_path.as_ref().to_path_buf();

        if !path.exists() {
            let parent = path.parent().unwrap();
            DirBuilder::new().recursive(true).create(parent).unwrap();
        }

        NoDb {
            map: DbMap::new(),
            list_map: DbListMap::new(),
            ser: Serializer::from(ser_method),
            path,
            policy,
            last_dump: Instant::now(),
        }
    }

    /// Loads a `NoDb` instance from a file.
    ///
    /// This method tries to load a DB from a file. Upon success an instance of `Ok(NoDb)` is returned,
    /// otherwise an `anyhow::Error` object is returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nodb::{NoDb, DumpPolicy, SerializationMethod};
    /// let nodb = NoDb::load("example.db", DumpPolicy::Auto, SerializationMethod::Json).unwrap();
    /// ```

    pub fn load<P: AsRef<Path>>(
        db_path: P,
        policy: DumpPolicy,
        ser_method: SerializationMethod,
    ) -> Result<Self> {
        let content = read(&db_path)?;
        let decrypted_content = B64.decrypt(content)?;
        let ser = Serializer::from(ser_method);
        let (map, list_map) = ser.deserialized_db(&decrypted_content)?;
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

    /// Dump the data to the file.
    ///
    /// Calling this method is necessary only if the DB is loaded or created with a dump policy other than
    /// [DumpPolicy::Auto](enum.DumpPolicy.html#variant.Auto), otherwise the data
    /// is dumped to the file upon every change unless the dump policy is
    /// [DumpPolicy::Never](enum.DumpPolicy.html#variant.Never).
    ///
    /// This method returns `Ok(())` if dump is successful, Or an `anyhow::Error` otherwise.

    pub fn dump(&mut self) -> Result<()> {
        if let DumpPolicy::Never = self.policy {
            return Ok(());
        }
        let data = self.ser.serialize_db(&self.map, &self.list_map)?;
        let encrypted_data = B64.encrypt(data);
        let tmp = format!(
            "{}.tmp.{}",
            self.path.to_str().unwrap_or("db"),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );

        write(&tmp, encrypted_data)?;
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

    /// Set a key-value pair.
    ///
    /// The key has to be a string but the value can be of any type that is serializable.
    /// That includes all primitive types, vectors, tuples, enums and every struct that
    /// has the `#[derive(Serialize, Deserialize)` attribute.
    ///
    /// This method returns `Ok(())` if set is successful, Or an `anyhow::Error`
    /// otherwise. An error is not likely to happen but may occur mostly in cases where this
    /// action triggers a DB dump (which is decided according to the dump policy).

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

    /// Get a value of a key.
    ///
    /// The key is always a string but the value can be of any type. It's the developer's
    /// responsibility to know the value type and give it while calling this method.
    /// If the key doesn't exist or if the type is wrong, `None` will be returned.
    /// Otherwise `Some(V)` will be returned.
    ///
    /// Since the values are stored in a serialized way the returned object is
    /// not a reference to the value stored in a DB but actually a new instance
    /// of it.

    pub fn get<K: AsRef<str>, V: DeserializeOwned>(&self, key: K) -> Option<V> {
        let key = key.as_ref();
        let res = self.map.get(key);
        if let Some(v) = res {
            self.ser.deserialize_data(v)
        } else {
            None
        }
    }

    /// Check if a key exists.
    ///
    /// This method returns `true` if the key exists and `false` otherwise.

    pub fn exists<K: AsRef<str>>(&self, key: K) -> bool {
        self.map.contains_key(key.as_ref()) || self.list_map.contains_key(key.as_ref())
    }

    /// Get a vector of all the keys in the DB.
    ///
    /// The keys returned in the vector are not references to the actual key string
    /// objects but rather a clone of them.

    pub fn get_all(&self) -> Vec<String> {
        [
            self.map.keys().cloned().collect::<Vec<String>>(),
            self.list_map.keys().cloned().collect::<Vec<String>>(),
        ]
        .concat()
    }

    /// Get the total number of keys in the DB.

    pub fn total_keys(&self) -> usize {
        self.map.iter().len() + self.list_map.iter().len()
    }

    /// Remove a key-value pair or a list from the DB.
    ///
    /// This methods returns `Ok(true)` if the key was found in the DB or `Ok(false)` if it wasn't found.
    /// It may also return `anyhow::Error` if key was found but removal failed.
    /// Removal error is not likely to happen but may occur mostly in cases where this action triggers a DB dump
    /// (which is decided according to the dump policy).

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

    /// Create a new list.
    ///
    /// This method just creates a new list, it doesn't add any elements to it.
    /// If another list or value is already set under this key, they will be overridden,
    /// meaning the new list will override the old list or value.
    ///
    /// Upon success, the method returns an object of type
    /// [NoDbExt](struct.NoDbExt.html) that enables to add
    /// items to the newly created list. Alternatively you can use [ladd()](#method.ladd)
    /// or [lextend()](#method.lextend) to add items to the list.

    pub fn lcreate<N: AsRef<str>>(&mut self, name: N) -> Result<NoDbExt> {
        let new_list = Vec::new();
        let name = name.as_ref();
        if self.map.contains_key(name) {
            self.map.remove(name);
        }
        self.list_map.insert(String::from(name), new_list);
        self.dumpdb()?;
        Ok(NoDbExt {
            db: self,
            list_name: name.to_string(),
        })
    }

    /// Check if a list exists.
    ///
    /// This method returns `true` if the list name exists and `false` otherwise.
    /// The difference between this method and [exists()](#method.exists) is that this methods checks only
    /// for lists with that name (key) and [exists()](#method.exists) checks for both values and lists.

    pub fn lexists<N: AsRef<str>>(&self, name: N) -> bool {
        self.list_map.contains_key(name.as_ref())
    }

    /// Add a single item to an existing list.
    ///
    /// As mentioned before, the lists are heterogeneous, meaning a single list can contain
    /// items of different types. That means that the item can be of any type that is serializable.
    /// That includes all primitive types, vectors, tuples and every struct that has the
    /// `#[derive(Serialize, Deserialize)` attribute.
    ///
    /// If the item was added successfully the method returns
    /// `Some(`[NoDbExt](struct.NoDbExt.html)`)` which enables to add more
    /// items to the list. Alternatively the method returns `None` if the list isn't found in the DB
    /// or if a failure happened while extending the list. Failures are not likely to happen but may
    /// occur mostly in cases where this action triggers a DB dump (which is decided according to the dump policy).

    pub fn ladd<K: AsRef<str>, V: Serialize>(&mut self, name: K, value: &V) -> Option<NoDbExt> {
        self.lextend(name, &[value])
    }

    /// Add multiple items to an existing list.
    ///
    /// As mentioned before, the lists are heterogeneous, meaning a single list can contain
    /// items of different types. That means that the item can be of any type that is serializable.
    /// That includes all primitive types, vectors, tuples and every struct that has the
    /// `#[derive(Serialize, Deserialize)` attribute.
    /// This method adds multiple items to the list, but since they're in a vector that means all
    /// of them are of the same type. Of course it doesn't mean that the list cannot contain items
    /// of other types as well, as you can see in the example below.
    ///
    /// If all items were added successfully the method returns
    /// `Some(`[NoDbExt](struct.NoDbExt.html)`)` which enables to add more
    /// items to the list. Alternatively the method returns `None` if the list isn't found in the DB
    /// or if a failure happened while extending the list. Failures are not likely to happen but may
    /// occur mostly in cases where this action triggers a DB dump (which is decided according to the dump policy).

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
                if self.dumpdb().is_err() {
                    let same_list = self.list_map.get_mut(name.as_ref()).unwrap();
                    same_list.truncate(orig_len);
                    return None;
                }
                Some(NoDbExt {
                    db: self,
                    list_name: name.as_ref().to_string(),
                })
            }
            None => None,
        }
    }

    /// Get an item of of a certain list in a certain position.
    ///
    /// This method takes a list name and a position inside the list
    /// and retrieves the item in this position. It's the developer's responsibility
    /// to know what is the correct type of the item and give it while calling this method.
    /// Since the item in the lists are stored in a serialized way the returned object
    /// is not a reference to the item stored in a DB but actually a new instance of it.
    /// If the list is not found in the DB or the given position is out of bounds
    /// of the list `None` will be returned. Otherwise `Some(V)` will be returned.

    pub fn lget<V: DeserializeOwned, N: AsRef<str>>(&self, name: N, pos: usize) -> Option<V> {
        match self.list_map.get(name.as_ref()) {
            Some(list) => match list.get(pos) {
                Some(val) => self.ser.deserialize_data::<V>(val),
                None => None,
            },
            None => None,
        }
    }

    /// Get the length of a list.
    ///
    /// If the list is empty or if it doesn't exist the value of 0 is returned.

    pub fn llen<N: AsRef<str>>(&self, name: N) -> usize {
        match self.list_map.get(name.as_ref()) {
            Some(list) => list.len(),
            None => 0,
        }
    }

    /// Remove a list.
    ///
    /// This method is somewhat similar to [rem()](#method.rem) but with 2 small differences:
    /// * This method only removes lists and not key-value pairs
    /// * The return value of this method is the number of items that were in
    ///   the list that was removed. If the list doesn't exist a value of zero (0) is
    ///   returned. In case of a failure an `anyhow::Error` is returned.
    ///   Failures are not likely to happen but may occur mostly in cases where this action triggers a
    ///   DB dump (which is decided according to the dump policy).

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

    /// Pop an item out of a list.
    ///
    /// This method takes a list name and a position inside the list, removes the
    /// item in this position and returns it to the user. It's the user's responsibility
    /// to know what is the correct type of the item and give it while calling this method.
    /// Since the item in the lists are stored in a serialized way the returned object
    /// is not a reference to the item stored in a DB but actually a new instance of it.
    ///
    /// If the list is not found in the DB or the given position is out of bounds no item
    /// will be removed and `None` will be returned. `None` may also be returned
    /// if removing the item fails, which may happen mostly in cases where this action
    /// triggers a DB dump (which is decided according to the dump policy).
    /// Otherwise the item will be removed and `Some(V)` will be returned.
    ///
    /// This method is very similar to [lrem_value()](#method.lrem_value), the only difference is that this
    /// methods returns the value and [lrem_value()](#method.lrem_value) returns only an indication whether
    /// the item was removed or not.

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

    /// Remove an item out of a list.
    ///
    /// This method takes a list name and a reference to a value, removes the first instance of the
    /// value if it exists in the list, and returns an indication whether the item was removed or not.
    ///
    /// If the list is not found in the DB or the given value isn't found in the list, no item will
    /// be removed and `Ok(false)` will be returned.
    /// If removing the item fails, which may happen mostly in cases where this action triggers
    /// a DB dump (which is decided according to the dump policy), an
    /// `anyhow::Error` is returned. Otherwise the item will be removed and `Ok(true)` will be returned.
    ///
    /// This method is very similar to [lpop()](#method.lpop), the only difference is that this
    /// methods returns an indication and [lpop()](#method.lpop) returns the actual item that was removed.

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

    /// Return an iterator over the keys and values in the DB.

    pub fn iter(&self) -> NoDbIter {
        NoDbIter {
            map_iter: self.map.iter(),
            ser: &self.ser,
        }
    }

    /// Return an iterator over the items in certain list.

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
