use nodb::{DumpPolicy, NoDb, Result, SerializationMethod};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

fn main() -> Result<()> {
    let mut trng = thread_rng();
    let mut db = NoDb::new(
        "./db/database.nodb",
        DumpPolicy::Auto,
        SerializationMethod::Cbor,
    );
    for _ in 0..50 {
        let random_id: usize = trng.gen_range(usize::MIN..usize::MAX);
        let user = User::new(random_id, "John Doe");
        let key = format!("user_{}", random_id);
        db.set(key, &user)?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct User {
    id: usize,
    name: String,
}

impl User {
    fn new(id: usize, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
        }
    }
}
