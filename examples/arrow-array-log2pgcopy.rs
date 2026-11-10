use std::io;
use std::process::ExitCode;

use io::Write;

use arrow_array::RecordBatch;
use arrow_array::record_batch;

use arrow_array::ArrayRef;

use arrow_array::BooleanArray;
use arrow_array::Int8Array;
use arrow_array::UInt8Array;

use rs_arrow_octets2pgcopy::ref2bools;
use rs_arrow_octets2pgcopy::ref2bytes;
use rs_arrow_octets2pgcopy::ref2chars;

use rs_arrow_octets2pgcopy::OctetLike;

fn io_record_batch() -> impl FnMut() -> Result<RecordBatch, io::Error> {
    || {
        record_batch!(
            ("id", Int64, [2, 3, 5, 7]),
            ("timestamp", Int64, [0, 0, 1, 2]),
            ("severity", UInt8, [9, 13, 17, 21]),
            ("is_active", Boolean, [false, false, false, false]),
            ("tag", Int8, [Some(42), None, Some(43), Some(44)]),
            (
                "body",
                Utf8,
                ["apt done", "apt failure", "i/o error", "OOM"]
            )
        )
        .map_err(io::Error::other)
    }
}

#[derive(Debug, PartialEq)]
enum OctetArray<'a> {
    Ignore,
    Bools(&'a BooleanArray),
    Chars(&'a Int8Array),
    Bytes(&'a UInt8Array),
}

impl<'a> OctetArray<'a> {
    fn to_writer<W>(&self, trusted_index: usize, wtr: &mut W) -> Result<(), io::Error>
    where
        W: Write,
    {
        match self {
            Self::Bools(l) => l.to_writer(trusted_index, wtr),
            Self::Chars(l) => l.to_writer(trusted_index, wtr),
            Self::Bytes(l) => l.to_writer(trusted_index, wtr),
            _ => Ok(()),
        }
    }
}

fn io_bat2stdout(rbat: RecordBatch) -> impl FnOnce() -> Result<(), io::Error> {
    move || {
        let rowcnt: usize = rbat.num_rows();
        let cols: &[ArrayRef] = rbat.columns();

        let mapd: Vec<OctetArray> = cols
            .iter()
            .map(|aref| {
                let obool = ref2bools(aref);
                let ochars = ref2chars(aref);
                let obytes = ref2bytes(aref);

                if let Some(b) = obool {
                    return OctetArray::Bools(b);
                }

                if let Some(c) = ochars {
                    return OctetArray::Chars(c);
                }

                if let Some(b) = obytes {
                    return OctetArray::Bytes(b);
                }

                OctetArray::Ignore
            })
            .filter(|i| !i.eq(&OctetArray::Ignore))
            .collect();

        let o = io::stdout();
        let mut ol = o.lock();

        let mut colsize: [u8; 2] = [0; 2];
        colsize[1] = mapd.len() as u8;

        for i in 0..rowcnt {
            ol.write_all(&colsize)?;
            for col in mapd.iter() {
                col.to_writer(i, &mut ol)?;
            }
        }
        Ok(())
    }
}

const PGCOPY_SIGN: &[u8] = b"PGCOPY\n\xff\r\n\0";
const PGCOPY_FLAGS: [u8; 4] = [0; 4];
const PGCOPY_EXT: [u8; 4] = [0; 4];

const PGCOPY_TRAILER: [u8; 2] = u16::to_be_bytes(0xffff);

fn io_header2stdout() -> impl FnOnce() -> Result<(), io::Error> {
    || {
        let o = io::stdout();
        let mut ol = o.lock();
        let mut buf: [u8; 19] = [0; 19];
        let src = PGCOPY_SIGN.iter().chain(&PGCOPY_FLAGS).chain(&PGCOPY_EXT);
        let z = buf.iter_mut().zip(src);
        for pair in z {
            let (dst, src) = pair;
            *dst = *src;
        }
        ol.write_all(&buf)?;
        ol.flush()
    }
}

fn io_trailer2stdout() -> impl FnOnce() -> Result<(), io::Error> {
    || {
        let o = io::stdout();
        let mut ol = o.lock();
        ol.write_all(&PGCOPY_TRAILER)?;
        ol.flush()
    }
}

fn io_main() -> impl FnOnce() -> Result<(), io::Error> {
    || {
        io_header2stdout()()?;
        let rbat: RecordBatch = io_record_batch()()?;
        io_bat2stdout(rbat)()?;
        io_trailer2stdout()()?;
        Ok(())
    }
}

fn sub() -> Result<(), io::Error> {
    io_main()()
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");
        ExitCode::FAILURE
    })
}
