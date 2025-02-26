//! # Crypto
//!
//! This module contains the `B64` struct which is used to encode and decode data using the `base64` algorithm.

use anyhow::Result;
use base64::{
    engine::{general_purpose::STANDARD, GeneralPurpose},
    Engine,
};

const STD: GeneralPurpose = STANDARD;

/// The `B64` struct is used to encrypt and decrypt data using the `base64` algorithm.
#[derive(Clone, Copy)]
pub struct B64;

impl B64 {
    /// Creates a new `B64` instance.
    pub const fn new() -> Self {
        Self {}
    }

    /// Encrypts the given data using the `base64` algorithm.
    pub fn encrypt<T: AsRef<[u8]>>(&self, data: T) -> String {
        STD.encode(data)
    }

    /// Decrypts the given data using the `base64` algorithm.
    pub fn decrypt<T: AsRef<[u8]>>(&self, data: T) -> Result<Vec<u8>> {
        Ok(STD.decode(data)?)
    }
}
