use anyhow::Result;
use bin::BinSer;
use cbor::CborSer;
use json::JsonSer;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use toml::TomlSer;

use crate::{DbListMap, DbMap};

mod bin;
mod cbor;
mod json;
mod toml;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SerializationMethod {
    #[default]
    Json,
    Bin,
    Cbor,
    Toml,
}

impl<T: Into<usize>> From<T> for SerializationMethod {
    fn from(value: T) -> Self {
        let value = value.into();
        match value {
            0 => SerializationMethod::Json,
            1 => SerializationMethod::Bin,
            2 => SerializationMethod::Cbor,
            3 => SerializationMethod::Toml,
            _ => SerializationMethod::Json,
        }
    }
}

impl Display for SerializationMethod {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:?}", self)
    }
}

pub trait SerializeMethod {
    fn serialize_data<T: Serialize>(&self, data: &T) -> Result<Vec<u8>>;
    fn serialize_db(&self, db_map: &DbMap, db_list_map: &DbListMap) -> Result<Vec<u8>>;
    fn deserialize_data<T: DeserializeOwned>(&self, data: &[u8]) -> Option<T>;
    fn deserialized_db(&self, ser_db: &[u8]) -> Result<(DbMap, DbListMap)>;
}

pub(crate) enum Serializer {
    Json(JsonSer),
    Bin(BinSer),
    Cbor(CborSer),
    Toml(TomlSer),
}

impl From<SerializationMethod> for Serializer {
    fn from(value: SerializationMethod) -> Self {
        match value {
            SerializationMethod::Json => Serializer::Json(JsonSer::new()),
            SerializationMethod::Bin => Serializer::Bin(BinSer::new()),
            SerializationMethod::Cbor => Serializer::Cbor(CborSer::new()),
            SerializationMethod::Toml => Serializer::Toml(TomlSer::new()),
        }
    }
}

impl SerializeMethod for Serializer {
    fn serialize_data<T: Serialize>(&self, data: &T) -> Result<Vec<u8>> {
        match self {
            Serializer::Json(json_ser) => json_ser.serialize_data(data),
            Serializer::Bin(bin_ser) => bin_ser.serialize_data(data),
            Serializer::Cbor(cbor_ser) => cbor_ser.serialize_data(data),
            Serializer::Toml(toml_ser) => toml_ser.serialize_data(data),
        }
    }
    fn serialize_db(&self, db_map: &DbMap, db_list_map: &DbListMap) -> Result<Vec<u8>> {
        match self {
            Serializer::Json(json_ser) => json_ser.serialize_db(db_map, db_list_map),
            Serializer::Bin(bin_ser) => bin_ser.serialize_db(db_map, db_list_map),
            Serializer::Cbor(cbor_ser) => cbor_ser.serialize_db(db_map, db_list_map),
            Serializer::Toml(toml_ser) => toml_ser.serialize_db(db_map, db_list_map),
        }
    }
    fn deserialize_data<T: DeserializeOwned>(&self, data: &[u8]) -> Option<T> {
        match self {
            Serializer::Json(json_ser) => json_ser.deserialize_data(data),
            Serializer::Bin(bin_ser) => bin_ser.deserialize_data(data),
            Serializer::Cbor(cbor_ser) => cbor_ser.deserialize_data(data),
            Serializer::Toml(toml_ser) => toml_ser.deserialize_data(data),
        }
    }
    fn deserialized_db(&self, ser_db: &[u8]) -> Result<(DbMap, DbListMap)> {
        match self {
            Serializer::Json(json_ser) => json_ser.deserialized_db(ser_db),
            Serializer::Bin(bin_ser) => bin_ser.deserialized_db(ser_db),
            Serializer::Cbor(cbor_ser) => cbor_ser.deserialized_db(ser_db),
            Serializer::Toml(toml_ser) => toml_ser.deserialized_db(ser_db),
        }
    }
}
