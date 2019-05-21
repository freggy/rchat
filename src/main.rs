extern crate bytebuffer;
extern crate byteorder;
extern crate crypto;
extern crate num_bigint;

pub mod netutils;

use netutils::VarInt;
use netutils::Strings;
use bytebuffer::ByteBuffer;
use std::net::TcpStream;
use std::io::{Write, Read};
use std::thread;
use std::fmt::Write as AWrite;
use self::crypto::digest::Digest;
use self::crypto::sha1::Sha1;
use num_bigint::BigInt;

fn main() -> std::io::Result<()> {
    let mut reader = TcpStream::connect("127.0.0.1:25565")?;
    let mut writer = reader.try_clone().expect("Could not clone");
    let mut buf = ByteBuffer::new();

    let t = thread::spawn(move || {
        let mut arr = vec![0; 5]; // allocate 5 bytes since varint are at max 5 bytes long
        loop {
            reader.read(&mut arr);

            let mut buf = ByteBuffer::from_bytes(&arr);

            let length = buf.read_var_int();
            let id = buf.read_var_int();

            println!("{}", length);
            println!("{}", id);

            let id_len = netutils::get_var_int_length(id);

            // length of packet = data length + length of id varint
            // that is why we subtract id_len from the over all length.
            // This way we get the length of the relevant packet data.
            let data_len = length - id_len;

            // grow vec to hold new data
            arr.resize(data_len as usize, 0);
            reader.read(&mut arr);
            buf.write_bytes(&mut arr);

            match id {
                0x01 => {
                    println!("Encryption Request");
                    let server_id = buf.read_string_utf8().unwrap();
                    let key_len = buf.read_var_int();
                    let key = buf.read_bytes(key_len as usize);
                    let token_length = buf.read_var_int();
                    let token = buf.read_bytes(token_length as usize);

                    println!("server id: {}", server_id);
                    println!("key: {:x?}", key);
                    println!("token: {:x?}", token);

                    // TOOD: encryption response
                }
                0x02 => {
                    println!("Login Success");
                    let uuid = buf.read_string_utf8().unwrap();
                    let name = buf.read_string_utf8();
                }
                _ => println!("Bad packet id")
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

fn create_mc_sha1(name: &str) -> String {
    let mut hasher = Sha1::new();
    let mut data = [0u8; 20];

    hasher.input_str(name);
    hasher.result(&mut data);

    let mut negative = (data[0] & 0x80) == 0x80;

    if negative {
        twos_complement(&mut data);
    }

    let mut hex = String::new();

    for i in data.iter() {
        write!(&mut hex, "{:x}", i).expect("Could not write to string");
    }

    if negative {
        hex.insert_str(0, "-")
    }

    hex
}

fn twos_complement(p: &mut [u8]) {
    let mut carry = true;
    for i in p.iter_mut().rev() {
        *i = !*i;
        if carry {
            carry = *i == 0xFFu8;
            *i += 1
        }
    }
}