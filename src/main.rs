use bytebuffer::ByteBuffer;

fn main() {



}

trait VarInt {
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
                break
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