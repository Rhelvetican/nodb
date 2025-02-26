use anyhow::Result;
use bin::BinSer;
use bit::BitSer;
use bson::BsonSer;
use cbor::CborSer;
use json::JsonSer;
use pot::PotSer;
use ron::RonSer;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};
use toml::TomlSer;

use crate::{DbListMap, DbMap};

mod bin;
mod bit;
mod bson;
mod cbor;
mod json;
mod pot;
mod ron;
mod toml;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SerializationMethod {
    #[default]
    Json,
    Bin,
    Cbor,
    Toml,
    Bit,
    Ron,
    Bson,
    Pot,
}

impl<T: Into<usize>> From<T> for SerializationMethod {
    fn from(value: T) -> Self {
        let value = value.into();
        match value {
            0 => SerializationMethod::Json,
            1 => SerializationMethod::Bin,
            2 => SerializationMethod::Cbor,
            3 => SerializationMethod::Toml,
            4 => SerializationMethod::Bit,
            5 => SerializationMethod::Ron,
            6 => SerializationMethod::Bson,
            7 => SerializationMethod::Pot,
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

pub(super) enum Serializer {
    Json(JsonSer),
    Bin(BinSer),
    Cbor(CborSer),
    Toml(TomlSer),
    Bit(BitSer),
    Ron(RonSer),
    Bson(BsonSer),
    Pot(PotSer),
}

impl From<SerializationMethod> for Serializer {
    fn from(value: SerializationMethod) -> Self {
        match value {
            SerializationMethod::Json => Serializer::Json(JsonSer::new()),
            SerializationMethod::Bin => Serializer::Bin(BinSer::new()),
            SerializationMethod::Cbor => Serializer::Cbor(CborSer::new()),
            SerializationMethod::Toml => Serializer::Toml(TomlSer::new()),
            SerializationMethod::Bit => Serializer::Bit(BitSer::new()),
            SerializationMethod::Ron => Serializer::Ron(RonSer::new()),
            SerializationMethod::Bson => Serializer::Bson(BsonSer::new()),
            SerializationMethod::Pot => Serializer::Pot(PotSer::new()),
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
            Serializer::Bit(bit_ser) => bit_ser.serialize_data(data),
            Serializer::Ron(ron_ser) => ron_ser.serialize_data(data),
            Serializer::Bson(bson_ser) => bson_ser.serialize_data(data),
            Serializer::Pot(pot_ser) => pot_ser.serialize_data(data),
        }
    }

    fn serialize_db(&self, db_map: &DbMap, db_list_map: &DbListMap) -> Result<Vec<u8>> {
        match self {
            Serializer::Json(json_ser) => json_ser.serialize_db(db_map, db_list_map),
            Serializer::Bin(bin_ser) => bin_ser.serialize_db(db_map, db_list_map),
            Serializer::Cbor(cbor_ser) => cbor_ser.serialize_db(db_map, db_list_map),
            Serializer::Toml(toml_ser) => toml_ser.serialize_db(db_map, db_list_map),
            Serializer::Bit(bit_ser) => bit_ser.serialize_db(db_map, db_list_map),
            Serializer::Ron(ron_ser) => ron_ser.serialize_db(db_map, db_list_map),
            Serializer::Bson(bson_ser) => bson_ser.serialize_db(db_map, db_list_map),
            Serializer::Pot(pot_ser) => pot_ser.serialize_db(db_map, db_list_map),
        }
    }

    fn deserialize_data<T: DeserializeOwned>(&self, data: &[u8]) -> Option<T> {
        match self {
            Serializer::Json(json_ser) => json_ser.deserialize_data(data),
            Serializer::Bin(bin_ser) => bin_ser.deserialize_data(data),
            Serializer::Cbor(cbor_ser) => cbor_ser.deserialize_data(data),
            Serializer::Toml(toml_ser) => toml_ser.deserialize_data(data),
            Serializer::Bit(bit_ser) => bit_ser.deserialize_data(data),
            Serializer::Ron(ron_ser) => ron_ser.deserialize_data(data),
            Serializer::Bson(bson_ser) => bson_ser.deserialize_data(data),
            Serializer::Pot(pot_ser) => pot_ser.deserialize_data(data),
        }
    }

    fn deserialized_db(&self, ser_db: &[u8]) -> Result<(DbMap, DbListMap)> {
        match self {
            Serializer::Json(json_ser) => json_ser.deserialized_db(ser_db),
            Serializer::Bin(bin_ser) => bin_ser.deserialized_db(ser_db),
            Serializer::Cbor(cbor_ser) => cbor_ser.deserialized_db(ser_db),
            Serializer::Toml(toml_ser) => toml_ser.deserialized_db(ser_db),
            Serializer::Bit(bit_ser) => bit_ser.deserialized_db(ser_db),
            Serializer::Ron(ron_ser) => ron_ser.deserialized_db(ser_db),
            Serializer::Bson(bson_ser) => bson_ser.deserialized_db(ser_db),
            Serializer::Pot(pot_ser) => pot_ser.deserialized_db(ser_db),
        }
    }
}
