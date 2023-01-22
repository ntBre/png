use std::io::{stdout, Write};

use crc::crc;
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
            ChunkType::Idat { data } => data,
            ChunkType::Iend => vec![],
        }
    }
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
    let mut png = Png::new();
    png.write_chunk(ChunkType::Ihdr {
        width: 400,
        height: 308,
        bit_depth: 1,
        color_type: 0,
        compression_method: 0,
        filter_method: 0,
        interlace_method: 0,
    });
    png.write_chunk(ChunkType::Idat {
        data: vec![128; 400 * 308],
    });
    png.write_chunk(ChunkType::Iend);
    stdout().write_all(&png.bytes).unwrap();
}
