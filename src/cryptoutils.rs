extern crate openssl;

use openssl::aes::{AesKey, KeyError, aes_ige};
use openssl::symm::Mode;
use openssl::rsa::{Rsa, Padding};

pub fn encrypt(out: &mut Vec<u8>, input: &Vec<u8>, raw: &[u8]) {
    match Rsa::public_key_from_der(raw) {
        Ok(e) => {
            e.public_encrypt(input, out, Padding::PKCS1);
        },
        Err(e) => {
            println!("{:?}", e)
        }
    }
}