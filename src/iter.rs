use std::{collections::hash_map::Iter as HashMapIter, slice::Iter as SliceIter};

use serde::de::DeserializeOwned;

use crate::ser::{SerializeMethod, Serializer};

/// Iterator object for iterating over keys and values in NoDb. Returned in [NoDb::iter()](struct.NoDb.html#method.iter)
pub struct NoDbIter<'a> {
    pub(crate) map_iter: HashMapIter<'a, String, Vec<u8>>,
    pub(crate) ser: &'a Serializer,
}

impl<'a> Iterator for NoDbIter<'a> {
    type Item = NoDbIterItem<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.map_iter.next() {
            Some((k, v)) => Some(NoDbIterItem {
                key: k,
                val: v,
                ser: self.ser,
            }),
            None => None,
        }
    }
}

/// The object returned in each iteration when iterating over keys and values in NoDb
pub struct NoDbIterItem<'a> {
    key: &'a str,
    val: &'a Vec<u8>,
    ser: &'a Serializer,
}

impl<'a> NoDbIterItem<'a> {
    /// Get the key

    pub fn get_key(&self) -> &str {
        self.key
    }

    /// Get the value of the key.
    ///
    /// The key is always a string but the value can be of any type. It's the user's
    /// responsibility to know the value type and give it while calling this method.
    /// If the key doesn't exist or if the type is wrong, `None` will be returned.
    /// Otherwise `Some(V)` will be returned.
    /// Since the values are stored in a serialized way the returned object is
    /// not a reference to the value stored in a DB but actually a new instance of it.
    /// The method returns `Some(V)` if deserialization succeeds or `None` otherwise.

    pub fn get_value<V>(&self) -> Option<V>
    where
        V: DeserializeOwned,
    {
        self.ser.deserialize_data::<V>(self.val)
    }
}

/// Iterator object for iterating over items in a NoDb list. Returned in [NoDb::liter()](struct.NoDb.html#method.liter)
pub struct NoDbListIter<'a> {
    pub(crate) list_iter: SliceIter<'a, Vec<u8>>,
    pub(crate) ser: &'a Serializer,
}

impl<'a> Iterator for NoDbListIter<'a> {
    type Item = NoDbListIterItem<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.list_iter.next() {
            Some(v) => Some(NoDbListIterItem {
                val: v,
                ser: self.ser,
            }),
            None => None,
        }
    }
}

/// The object returned in each iteration when iterating over a NoDb list
pub struct NoDbListIterItem<'a> {
    val: &'a Vec<u8>,
    ser: &'a Serializer,
}

impl<'a> NoDbListIterItem<'a> {
    /// Get the item in the current position.
    ///
    /// This method retrieves the item in the current position. It's the user's responsibility
    /// to know what is the correct type of the item and give it while calling this method.
    /// Since the item in the lists are stored in a serialized way the returned object
    /// is not a reference to the item stored in a DB but actually a new instance of it.
    /// The method returns `Some(V)` if deserialization succeeds or `None` otherwise.
    ///
    pub fn get_item<V: DeserializeOwned>(&self) -> Option<V> {
        self.ser.deserialize_data(self.val)
    }
}
