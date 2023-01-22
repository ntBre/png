use std::io::{stdout, Write};

use crc::crc;
use flate2::{
    write::{DeflateEncoder, ZlibEncoder},
    Compression,
};
mod crc;

enum ChunkType {
    Ihdr {
        width: u32,
        height: u32,
        bit_depth: u8,
        color_type: u8,
        compression_method: u8,
        filter_method: u8,
        interlace_method: u8,
    },
    Idat {
        data: Vec<u8>,
    },
    Iend,
}

impl ChunkType {
    fn header(&self) -> Vec<u8> {
        match self {
            ChunkType::Ihdr { .. } => vec![73, 72, 68, 82],
            ChunkType::Iend => vec![73, 69, 78, 68],
            ChunkType::Idat { .. } => vec![73, 68, 65, 84],
        }
    }

    fn data(self) -> Vec<u8> {
        match self {
            ChunkType::Ihdr {
                width,
                height,
                bit_depth,
                color_type,
                compression_method,
                filter_method,
                interlace_method,
            } => {
                let mut ret = bytes(width).to_vec();
                ret.extend(bytes(height));
                ret.extend([
                    bit_depth,
                    color_type,
                    compression_method,
                    filter_method,
                    interlace_method,
                ]);
                ret
            }
            ChunkType::Idat { data } => {
                let mut buf = Vec::with_capacity(data.len());
                // zlib compression method and flags, original method was 7 but
                // imagemagick uses 6
                let cmf = 6 << 4 | 8;
                // default compression algorithm = 2, no preset dictionary = 0.
                // imagemagick uses compression algorithm 3 -> maximum
                // compression
                let mut flg = 3 << 6 | 0 << 5;
                let v = cmf as u16 * 256 + flg;
                let fcheck = 31 - v % 31;
                flg |= fcheck;
                eprintln!("{:x}", flg);
                buf.extend([cmf as u8, flg as u8]);
                let adler = adler32(&data);
                let mut e =
                    DeflateEncoder::new(Vec::new(), Compression::default());
                e.write_all(&data).unwrap();
                let compressed = e.finish().unwrap();
                buf.extend(compressed);
                eprintln!("{:02x?}", bytes(adler));
                buf.extend(bytes(adler));
                buf
            }
            ChunkType::Iend => vec![],
        }
    }
}

fn adler32(bytes: &[u8]) -> u32 {
    const ADLER: u32 = 65521;
    let mut s1 = 1;
    let mut s2 = 0;
    for b in bytes {
        s1 = (s1 + *b as u32) % ADLER;
        s2 = (s2 + s1) % ADLER;
    }
    s2 * 65536 + s1
}

struct Png {
    bytes: Vec<u8>,
}

fn bytes(d: u32) -> [u8; 4] {
    d.to_be_bytes()
}

impl Png {
    /// return a [Png] initialized with the PNG signature
    fn new() -> Self {
        Self {
            bytes: vec![137, 80, 78, 71, 13, 10, 26, 10],
        }
    }

    /// write `chunk_type` with data field `data` to `self`
    fn write_chunk(&mut self, chunk_type: ChunkType) {
        // get the header first, so we can move out of chunk_type for data
        let header = chunk_type.header();
        // write the length of the chunk
        let data = chunk_type.data();
        let len = data.len() as u32;
        let mut buf = bytes(len).to_vec();
        // write the header for chunk_type
        buf.extend(header);
        // write the chunk data
        buf.extend(data);
        // compute the CRC for the chunk (minus length field)
        let crc = crc(&buf[4..], buf.len() - 4);
        buf.extend(bytes(crc));
        // store in self
        self.bytes.extend(buf);
    }
}

fn main() {
    let w = 1920;
    let h = 1080;
    let mut png = Png::new();
    png.write_chunk(ChunkType::Ihdr {
        width: w,
        height: h,
        bit_depth: 8,
        color_type: 2,
        compression_method: 0,
        filter_method: 0,
        interlace_method: 0,
    });
    let mut data = Vec::new();
    for _row in 0..h {
        data.push(0);
        for _col in 0..h {
            data.push(3);
            data.push(3);
            data.push(3);
        }
    }
    png.write_chunk(ChunkType::Idat { data });
    png.write_chunk(ChunkType::Iend);
    stdout().write_all(&png.bytes).unwrap();
}
