use bytebuffer::ByteBuffer;
use std::string::{String, FromUtf8Error};
use std::io::Read;
use std::fmt::Error;


pub trait Strings {
    fn read_string_utf8(&mut self) -> Result<String, FromUtf8Error>;
    fn write_string_utf8(&mut self, string: &str);
}

impl Strings for ByteBuffer {

    fn read_string_utf8(&mut self) -> Result<String, FromUtf8Error> {
        let len = self.read_var_int();
        String::from_utf8(self.read_bytes(len as usize))
    }

    fn write_string_utf8(&mut self, string: &str) {
        self.write_var_int(string.len() as i32);
        self.write_bytes(string.as_bytes())
    }
}



pub trait VarInt {
    fn read_var_int(&mut self) -> i32;
    fn write_var_int(&mut self, num: i32);
}

impl VarInt for ByteBuffer {
    fn read_var_int(&mut self) -> i32 {
        let mut num_read = 0;
        let mut result = 0;
        let mut byte_read = 0;

        loop {
            byte_read = self.read_u8();
            let val = (byte_read & 0x7F) as i32;
            result |= val << (7 * num_read);
            num_read += 1;

            if num_read > 5 {
                break;
            }

            if (byte_read & 0x80) == 0 {
                break;
            }
        }

        result
    }

    fn write_var_int(&mut self, num: i32) {
        let mut val = num;
        loop {
            let mut temp = val & 0x7F;
            val >>= 7;
            if val != 0 {
                temp |= 0x80
            }
            self.write_u8(temp as u8);
            if val == 0 {
                break;
            }
        }
    }
}

pub fn get_var_int_length(val: i32) -> i32 {
    for i in 1..5 {
        if (val & -1 << i * 7) == 0 {
            return i;
        }
    }
    5
}