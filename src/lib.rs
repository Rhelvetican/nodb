//! # NoDb
//! ======
//!
//! NoDb is a simple key-value store that stores data in a single file. It is designed to be used in
//! small projects where a full-fledged database is not required. It is based on [PickleDB-RS](https://github.com/seladb/pickledb-rs/)
//!
//! ## Features
//! - **Simple**: NoDb is simple to use, similar to PickleDB.
//! - **Fast**: NoDb is fast, as it stores data in memory and writes to disk only when required.
//! - **Lightweight**: NoDb is lightweight, with only a few dependencies.
//! - **Serialization**: NoDb supports different serialization methods with Serde.
//! - **Encrypted**: NoDb supports encryption of data (Currently uses Base64 Encryption).

pub use anyhow::Result;
use std::collections::HashMap;

type DbMap = HashMap<String, Vec<u8>>;
type DbListMap = HashMap<String, Vec<Vec<u8>>>;

pub use self::{
    ext::NoDbExt,
    iter::{NoDbIter, NoDbIterItem, NoDbListIter, NoDbListIterItem},
    nodb::{DumpPolicy, NoDb},
    ser::SerializationMethod,
};

pub mod prelude {
    pub use crate::{NoDb, NoDbExt, SerializationMethod};
}

mod crypto;
mod ext;
mod iter;
mod nodb;
mod ser;
