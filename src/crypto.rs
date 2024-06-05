use anyhow::Result;
use base64::{
    engine::{general_purpose::STANDARD, GeneralPurpose},
    Engine,
};

const STD: GeneralPurpose = STANDARD;

pub struct B64 {}

impl B64 {
    pub const fn new() -> Self {
        Self {}
    }
    pub fn encrypt<T: AsRef<[u8]>>(&self, data: T) -> String {
        STD.encode(data)
    }
    pub fn decrypt<T: AsRef<[u8]>>(&self, data: T) -> Result<Vec<u8>> {
        Ok(STD.decode(data)?)
    }
}
