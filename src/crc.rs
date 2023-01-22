//! implementation of the Cyclic Redundancy Check (CRC) employed for PNG chunks,
//! adapted from Annex D of the [PNG
//! specificiation](https://www.w3.org/TR/2003/REC-PNG-20031110/#D-CRCAppendix)

const CRC_TABLE: [u32; 256] = make_crc_table();

const fn make_crc_table() -> [u32; 256] {
    let mut crc_table = [0; 256];
    let mut n = 0;
    while n < 256 {
        let mut c = n as u32;
        let mut k = 0;
        while k < 8 {
            if c & 1 != 0 {
                c = 0xedb88320 ^ (c >> 1);
            } else {
                c >>= 1;
            }
            k += 1;
        }
        crc_table[n] = c;
        n += 1;
    }
    crc_table
}

fn update_crc(crc: u32, buf: &[u8], len: usize) -> u32 {
    let mut c = crc;
    for n in 0..len {
        c = CRC_TABLE[((c ^ buf[n] as u32) & 0xff) as usize] ^ (c >> 8);
    }
    c
}

pub(crate) fn crc(buf: &[u8], len: usize) -> u32 {
    update_crc(0xffffffff, buf, len) ^ 0xffffffff
}

#[cfg(test)]
mod tests {
    #[test]
    fn crc() {
        let b = [
            0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x01, 0x77, 0x00, 0x00, 0x01,
            0x68, 0x08, 0x06, 0x00, 0x00, 0x00,
        ];
        let got = super::crc(&b, 17);
        assert_eq!(got, 0xac40bbb0);
    }
}
