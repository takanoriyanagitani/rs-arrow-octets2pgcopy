use std::io;

use io::Write;

use arrow_array::Array;
use arrow_array::ArrayRef;
use arrow_array::cast::AsArray;

use arrow_array::BooleanArray;

use arrow_array::Int8Array;
use arrow_array::UInt8Array;

pub const NULL_BYTES: [u8; 4] = u32::to_be_bytes(0xffff_ffff);

pub trait OctetLike: Array {
    fn to_byte(&self, trusted_index: usize) -> u8;

    fn to_writer<W>(&self, trusted_index: usize, wtr: &mut W) -> Result<(), io::Error>
    where
        W: Write,
    {
        let is_null: bool = self.is_null(trusted_index);
        if is_null {
            wtr.write_all(&NULL_BYTES)?;
            return Ok(());
        }

        let val: u8 = self.to_byte(trusted_index);
        let mut buf: [u8; 5] = [0; 5];
        buf[3] = 1;
        buf[4] = val;
        wtr.write_all(&buf)
    }
}

impl OctetLike for &Int8Array {
    fn to_byte(&self, trusted_index: usize) -> u8 {
        self.value(trusted_index) as u8
    }
}

impl OctetLike for &UInt8Array {
    fn to_byte(&self, trusted_index: usize) -> u8 {
        self.value(trusted_index)
    }
}

impl OctetLike for &BooleanArray {
    fn to_byte(&self, trusted_index: usize) -> u8 {
        match self.value(trusted_index) {
            true => 1,
            false => 0,
        }
    }
}

pub fn ref2bools(r: &ArrayRef) -> Option<&BooleanArray> {
    r.as_boolean_opt()
}

pub fn ref2chars(r: &ArrayRef) -> Option<&Int8Array> {
    r.as_primitive_opt()
}

pub fn ref2bytes(r: &ArrayRef) -> Option<&UInt8Array> {
    r.as_primitive_opt()
}
