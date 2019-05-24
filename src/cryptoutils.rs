extern crate openssl;
extern crate cfb8;
extern crate aes;

use openssl::aes::{AesKey, KeyError, aes_ige};
use openssl::symm::{Mode, Cipher, Crypter};
use openssl::rsa::{Rsa, Padding};
use std::io::Write;

use aes::Aes128;
use cfb8::Cfb8;
use cfb8::stream_cipher::{NewStreamCipher, StreamCipher};


pub fn encrypt(out: &mut Vec<u8>, input: &Vec<u8>, raw: &[u8]) {
    match Rsa::public_key_from_der(raw) {
        Ok(e) => {
            e.public_encrypt(input, out, Padding::PKCS1);
        }
        Err(e) => {
            println!("{:?}", e)
        }
    }
}

pub fn decrypt(cipher: &mut Crypter, input: &mut Vec<u8>) {
    let mut out = vec![0u8; input.len()];
    cipher.update(&input, &mut out);
    cipher.finalize(&mut out);
    *input = out;
}