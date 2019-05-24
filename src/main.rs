extern crate bytebuffer;
extern crate byteorder;
extern crate crypto;
extern crate rand;
extern crate reqwest;

#[macro_use]
extern crate serde_json;

pub mod netutils;
pub mod mojang;
pub mod cryptoutils;


extern crate openssl;

use openssl::aes::{AesKey, KeyError, aes_ige};
use openssl::symm::{Mode, Cipher, Crypter};
use openssl::rsa::{Rsa, Padding};

use netutils::VarInt;
use netutils::Strings;
use bytebuffer::ByteBuffer;
use std::net::TcpStream;
use std::io::{Write, Read};
use std::thread;
use openssl::pkey::PKey;
use self::crypto::digest::Digest;
use self::crypto::sha1::Sha1;
use rand::{thread_rng, Rng};
use std::fs::{File, read};
use reqwest::Client;

use openssl::symm;
use openssl::symm::Mode::Decrypt;


fn dec(c: &mut Crypter, data: Vec<u8>) {
    let mut out = vec![0u8; data.len()];

    c.update(&data, &mut out);

    let mut bnuf = ByteBuffer::from_bytes(&out);

    let l = bnuf.read_var_int();
    let id = bnuf.read_var_int();

    println!("{}", id);
}

fn main() -> std::io::Result<()> {
    // 22
    // cd2f37be07327e

    // 980b56c86279259081d0905442e5bb9c

    /*
    let data = hex::decode("9d506aff6b34171bf33c2aad5da72e42366ff4789d3192a0ed51383190aea8e84e5f87").unwrap();
    let secret = hex::decode("980b56c86279259081d0905442e5bb9c").unwrap();
    let mut c = symm::Crypter::new(symm::Cipher::aes_128_cfb8(), Decrypt, &secret, Some(&secret)).unwrap();


    dec(&mut c, hex::decode("52ccbfcf5d5965ccde1486226557d7f38b5b16760b5dec01a211e025bf747a978d897fa282fd23de19d5e65237ece3").unwrap());
    dec(&mut c, hex::decode("c5d0e9f5d59e7f5bff76bbafa7e3d1b46c3982").unwrap());

*/
    let mut reader = TcpStream::connect("127.0.0.1:25565")?;
    let mut writer = reader.try_clone().expect("Could not clone");
    let mut buf = ByteBuffer::new();


    create_handshake_data(&mut buf, 47, "127.0.0.1", 25565, 2);
    send_packet(&mut writer, &mut buf, false, 0x00);

    create_login_start_data(&mut buf, "freggyy");
    send_packet(&mut writer, &mut buf, false, 0x00);


    {
        let mut arr = vec![0u8; 16]; // allocate 5 bytes since varint are at max 5 bytes long
        let mut encrypted = false;
        let mut secret = vec![0u8; 16];
        let mut state = "LOGIN";
        let mut cipher: Option<Crypter> = Option::None;

        loop {
            reader.read(&mut arr);


            if encrypted == true {
                match cipher.as_mut() {
                    Some(c) => {
                        let mut out = vec![0u8; arr.len()];
                        c.update(&arr, &mut out);
                        let mut buf2 = ByteBuffer::from_bytes(&out);
                        buf2.read_var_int();
                        let id2 = buf2.read_var_int();
                        //println!("id: {}", id2);
                    },
                    None => println!("NONE")
                }
            }

            let mut buf = ByteBuffer::from_bytes(&arr);
            let len = buf.read_var_int();
            let id = buf.read_var_int();

            // println!("id: {}", id);


            match id {
                0x01 => {

                    if state == "LOGIN" {

                        alloc(&mut reader, &mut arr, &mut buf, &mut None, len as usize, false);

                        println!("Received encryption request");
                        let server_id = buf.read_string_utf8().unwrap();
                        let key_len = buf.read_var_int();
                        let key = buf.read_bytes(key_len as usize);
                        let token_len = buf.read_var_int();
                        let token = buf.read_bytes(token_len as usize);


                        //File::create("test.der").unwrap().write_all(&mut lol);

                        let client = Client::new();

                        match mojang::auth(&client, "qqontrol@web.de", "YEjuloUeiuc4H2p5Styv3jASmbyPZu") {
                            Ok(t) => {
                                let mut rng = thread_rng();
                                rng.fill(secret.as_mut_slice());
                                mojang::join_server(&client, t, server_id, &secret, &key).expect("Could not join.");

                                println!("{:?} ", hex::encode(&secret));

                                cipher = Some(symm::Crypter::new(symm::Cipher::aes_128_cfb8(), Decrypt, &secret, Some(&secret)).unwrap());

                                buf.clear(); // re-use

                                create_encryption_response_data(&mut buf, key, &secret, token);
                                encrypted = true;
                                state = "PLAY";
                                send_packet(&mut reader, &mut buf, false, 0x01);
                            }
                            Err(e) => {
                                println!("Error while authentication: {:?}", e);
                                return Ok(());
                            }
                        }
                        println!("Sending encryption response");
                    }
                }
                0x02 => {
                    if state == "LOGIN" {
                        println!("Login successful!");
                        state = "PLAY";
                    }
                    else {
                        println!("lul")
                        /*
                        alloc(&mut reader, &mut arr, &mut buf, &secret, len as usize, true);
                        let string = buf.read_string_utf8().unwrap();
                        println!("{:?}", string);*/
                    }
                }
                e => lol()
            }
        }
    }


    Ok(())
}

fn alloc(reader: &mut TcpStream, data: &mut Vec<u8>, buf: &mut ByteBuffer, cipher: &mut Option<Crypter>, len: usize, decrypt: bool) {
    if len > data.len() {
        data.resize(len, 0);
        buf.resize(len);
    }

    if decrypt {
        match cipher.as_mut() {
            Some(c) => {
                cryptoutils::decrypt(c, data);
            },
            None => println!("NONE")
        }
    }

    reader.read(data);
    buf.write_bytes(data);
}



fn send_packet(tcp: &mut TcpStream, data: &mut ByteBuffer, encrypted: bool, id: i32) {
    let mut buf = ByteBuffer::new();
    buf.write_var_int(netutils::get_var_int_length(id) + (data.len() as i32));
    buf.write_var_int(id);
    buf.write_bytes(data.to_bytes().as_slice());
    tcp.write_all(buf.to_bytes().as_slice());
}

fn create_handshake_data(buf: &mut ByteBuffer, protocol_version: i32, ip: &str, port: u16, state: i32) {
    buf.clear();
    buf.write_var_int(protocol_version);
    buf.write_string_utf8(ip);
    buf.write_u16(port);
    buf.write_var_int(state);
}

fn create_login_start_data(buf: &mut ByteBuffer, name: &str) {
    buf.clear();
    buf.write_string_utf8(name)
}
fn lol() {

}

fn create_encryption_response_data(buf: &mut ByteBuffer, key: Vec<u8>, secret: &Vec<u8>, token: Vec<u8>) {
    buf.clear();
    let mut sec_secret = vec![0u8; 128];
    let mut sec_token = vec![0u8; 128];

    cryptoutils::encrypt(&mut sec_secret, secret, &key);
    cryptoutils::encrypt(&mut sec_token, &token, &key);

    buf.write_var_int(sec_secret.len() as i32);
    buf.write_bytes(&sec_secret);

    buf.write_var_int(sec_token.len() as i32);
    buf.write_bytes(&sec_token);
}
