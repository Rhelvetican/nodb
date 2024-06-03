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

mod ext;
mod iter;
mod nodb;
mod ser;
