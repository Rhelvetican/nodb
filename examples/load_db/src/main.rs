use anyhow::Result;
use nodb::{DumpPolicy, NoDb, SerializationMethod};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

fn main() -> Result<()> {
    let db = NoDb::load(
        "./db/database.nodb",
        DumpPolicy::Never,
        SerializationMethod::Cbor,
    )?;
    let keys = db.get_all();
    for key in keys {
        let user: User = db.get(&key).unwrap();
        println!("{}", user);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct User {
    id: usize,
    name: String,
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "[id:{}]: user_{}", self.id, self.name)
    }
}
