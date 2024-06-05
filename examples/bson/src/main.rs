use anyhow::Result;
use nodb::{DumpPolicy, NoDb, SerializationMethod};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

fn main() -> Result<()> {
    let mut trng = thread_rng();
    let mut db = NoDb::new(
        "./db/nosql.nodb",
        DumpPolicy::Auto,
        SerializationMethod::Cbor,
    );
    for _ in 0..50 {
        let random_id: isize = trng.gen_range(0..isize::MAX);
        let user = User::new(random_id, "John Doe");
        let key = format!("user_{}", random_id);
        db.set(key, &user)?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct User {
    id: isize,
    name: String,
}

impl User {
    fn new(id: isize, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
        }
    }
}
