use std::collections::HashMap;

type DbMap = HashMap<String, Vec<u8>>;
type DbListMap = HashMap<String, Vec<Vec<u8>>>;

mod ext;
mod iter;
mod nodb;
mod ser;
