extern crate bytebuffer;
extern crate byteorder;
extern crate crypto;
extern crate rand;

#[macro_use]
extern crate serde_json;

pub mod netutils;
pub mod mojang;
pub mod cryptoutils;

extern crate openssl;

use openssl::aes::{AesKey, KeyError, aes_ige};
use openssl::symm::Mode;
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
use std::fs::File;

fn main() -> std::io::Result<()> {
    let mut reader = TcpStream::connect("127.0.0.1:25565")?;
    let mut writer = reader.try_clone().expect("Could not clone");
    let mut buf = ByteBuffer::new();

    let t = thread::spawn(move || {
        let mut arr = vec![0; 2048]; // allocate 5 bytes since varint are at max 5 bytes long

        loop {
            reader.read(&mut arr);


            let mut buf = ByteBuffer::from_bytes(&arr);

            let length = buf.read_var_int();
            let id = buf.read_var_int();

            /*
            println!("{}", length);
            println!("{}", id);

            let id_len = netutils::get_var_int_length(id);

            // length of packet = data length + length of id varint
            // that is why we subtract id_len from the over all length.
            // This way we get the length of the relevant packet data.
            let data_len = length - id_len;

            // grow vec to hold new data
            arr.resize(1024, 0);
            reader.read(&mut arr);
            buf.write_bytes(&mut arr);*/

            match id {
                0x01 => {
                    println!("Received encryption request");
                    let server_id = buf.read_string_utf8().unwrap();
                    let key_len = buf.read_var_int();
                    let key = buf.read_bytes(key_len as usize);
                    let token_len = buf.read_var_int();
                    let token = buf.read_bytes(token_len as usize);

                    //File::create("test.der").unwrap().write_all(&mut lol);

                    match mojang::auth("<user>", "<password>") {
                        Ok(t) => {
                            let mut rng = thread_rng();
                            let mut secret = vec![0u8; 16];
                            rng.fill(secret.as_mut_slice());

                            mojang::join_server(t, server_id, &secret, &key).expect("Could not join.");

                            let mut sec_secret = vec![0u8; 128];
                            cryptoutils::encrypt(&mut sec_secret, &secret, &key);

                            let mut sec_token = vec![0u8; 128];
                            cryptoutils::encrypt(&mut sec_token, &token, &key);

                            let mut data = ByteBuffer::new();

                            data.write_var_int(sec_secret.len() as i32);
                            data.write_bytes(&sec_secret);

                            data.write_var_int(sec_token.len() as i32);
                            data.write_bytes(&sec_token);

                            let mut packet = ByteBuffer::new();

                            packet.write_var_int(1 + data.len() as i32);
                            packet.write_var_int(0x01);
                            packet.write_bytes(data.to_bytes().as_slice());


                            reader.write_all(packet.to_bytes().as_slice()).unwrap();
                            reader.flush();
                            println!("lil")
                        }
                        Err(e) => {
                            println!("Error while authenitcating: {:?}", e);
                            return;
                        }
                    }

                    println!("Sending encryption response");

                    // TOOD: encryption response
                }
                0x02 => {
                    println!("Login successful!");
                }
                e => println!("Bad packet id: {}", e)
            }
        }
    });

    create_handshake_data(&mut buf, 47, "127.0.0.1", 25565, 2);
    send_packet(0x00, &mut buf, &mut writer);

    create_login_start_data(&mut buf, "freggyy");
    send_packet(0x00, &mut buf, &mut writer);


    t.join();
    Ok(())
}

fn send_packet(id: i32, data: &mut ByteBuffer, tcp: &mut TcpStream) {
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