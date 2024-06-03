use std::{collections::hash_map::Iter as HashMapIter, slice::Iter as SliceIter};

use serde::de::DeserializeOwned;

use crate::ser::{SerializeMethod, Serializer};

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

pub struct NoDbIterItem<'a> {
    key: &'a str,
    val: &'a Vec<u8>,
    ser: &'a Serializer,
}

impl<'a> NoDbIterItem<'a> {
    pub fn get_key(&self) -> &str {
        self.key
    }
    pub fn get_value<V>(&self) -> Option<V>
    where
        V: DeserializeOwned,
    {
        self.ser.deserialize_data::<V>(self.val)
    }
}

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

pub struct NoDbListIterItem<'a> {
    val: &'a Vec<u8>,
    ser: &'a Serializer,
}

impl<'a> NoDbListIterItem<'a> {
    pub fn get_item<V: DeserializeOwned>(&self) -> Option<V> {
        self.ser.deserialize_data(self.val)
    }
}
