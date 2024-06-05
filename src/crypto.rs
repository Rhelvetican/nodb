//! # Crypto
//!
//! This module contains the `B64` struct which is used to encrypt and decrypt data using the `base64` algorithm.

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
