extern crate bytebuffer;
extern crate byteorder;

pub mod netutils;

use netutils::VarInt;
use netutils::Strings;
use bytebuffer::ByteBuffer;
use std::net::TcpStream;
use std::io::{Write, Read};
use std::thread;
use std::sync::{Arc, RwLock};
use core::borrow::BorrowMut;

fn main() -> std::io::Result<()> {
    let mut reader = TcpStream::connect("127.0.0.1:25565")?;
    let mut writer = reader.try_clone().expect("Could not clone");

    let mut packet = ByteBuffer::new();
    let mut data = ByteBuffer::new();

    let t = thread::spawn(move || {
        let mut arr = [0u8; 1024];
        loop {
            reader.read(&mut arr);
            let mut buf = ByteBuffer::from_bytes(&arr);

            let length = buf.read_var_int();
            let id = buf.read_var_int();
            println!("{}", length);
            println!("{}", id);
        }
    });


    // HANDSHAKE

    data.write_var_int(47);
    data.write_string_utf8("127.0.0.1");
    data.write_u16(25565);
    data.write_var_int(2);

    packet.write_var_int(1 + (data.len() as i32));
    packet.write_var_int(0x00);
    packet.write_bytes(data.to_bytes().as_slice());

    writer.write_all(packet.to_bytes().as_slice());


    // LOGIN START

    let mut packet = ByteBuffer::new();
    let mut data = ByteBuffer::new();

    data.write_string_utf8("freggyy");

    packet.write_var_int(1 + (data.len() as i32));
    packet.write_var_int(0x00);
    packet.write_bytes(data.to_bytes().as_slice());

    writer.write_all(packet.to_bytes().as_slice());

    t.join();

    Ok(())
}




