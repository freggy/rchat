extern crate bytebuffer;
extern crate byteorder;

pub mod netutils;

use netutils::VarInt;
use netutils::Strings;
use bytebuffer::ByteBuffer;
use std::net::TcpStream;
use std::io::{Write, Read};
use std::thread;

fn main() -> std::io::Result<()> {
    let mut reader = TcpStream::connect("127.0.0.1:25565")?;
    let mut writer = reader.try_clone().expect("Could not clone");
    let mut buf = ByteBuffer::new();

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