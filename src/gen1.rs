const TABLE1: [usize; 16] = [
    1, 3, 7, 15, 31, 63, 127, 255, 511, 1023, 2047, 4095, 8191, 16383, 32767, 65535,
];

#[rustfmt::skip]
const TABLE2: [[usize; 16]; 4] = [
    [0x0, 0x1, 0x3, 0x2, 0x7, 0x6, 0x4, 0x5, 0xf, 0xe, 0xc, 0xd, 0x8, 0x9, 0xb, 0xa],
    [0xf, 0xe, 0xc, 0xd, 0x8, 0x9, 0xb, 0xa, 0x0, 0x1, 0x3, 0x2, 0x7, 0x6, 0x4, 0x5], // prev ^ 0xf
    [0x0, 0x8, 0xc, 0x4, 0xe, 0x6, 0x2, 0xa, 0xf, 0x7, 0x3, 0xb, 0x1, 0x9, 0xd, 0x5],
    [0xf, 0x7, 0x3, 0xb, 0x1, 0x9, 0xd, 0x5, 0x0, 0x8, 0xc, 0x4, 0xe, 0x6, 0x2, 0xa], // prev ^ 0xf
];

const TILESIZE: usize = 8;

struct BitStream<'a> {
    data: &'a [u8],
    bit_offset: usize,
    byte_offset: usize,
}

impl<'a> BitStream<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            bit_offset: 0,
            byte_offset: 0,
        }
    }

    fn next(&mut self) -> bool {
        let bit = ((self.data[self.byte_offset]) >> (7 - self.bit_offset)) & 1;

        self.bit_offset += 1;
        if self.bit_offset == 8 {
            self.bit_offset = 0;
            self.byte_offset += 1;
        }

        bit != 0
    }

    fn read_int(&mut self, mut count: usize) -> usize {
        let mut n = 0;

        while count > 0 {
            n <<= 1;
            n |= self.next() as usize;
            count -= 1;
        }

        n
    }
}

struct Decompressor<'a> {
    bs: BitStream<'a>,
    sizex: usize,
    sizey: usize,
    size: usize,
}

impl<'a> Decompressor<'a> {
    fn new(data: &'a [u8]) -> Self {
        let mut bs = BitStream::new(data);

        let sizex = bs.read_int(4) * TILESIZE;
        let sizey = bs.read_int(4);

        let size = sizex * sizey;

        Self {
            bs,
            sizex,
            sizey,
            size,
        }
    }

    fn decompress(&mut self) -> Vec<u8> {
        let mut rams = [vec![], vec![]];

        let r1 = self.bs.next() as usize;
        let r2 = r1 ^ 1;

        self.fillram(&mut rams[r1]);

        let mode = if self.bs.next() {
            if self.bs.next() {
                2
            } else {
                1
            }
        } else {
            0
        };

        self.fillram(&mut rams[r2]);

        bitgroups_to_bytes(&mut rams[0]);
        bitgroups_to_bytes(&mut rams[1]);

        match mode {
            0 => {
                self.decode(&mut rams[0]);
                self.decode(&mut rams[1]);
            }
            1 => {
                self.decode(&mut rams[r1]);
                let r1_ram = rams[r1].clone();
                self.xor(&r1_ram, &mut rams[r2]);
            }
            2 => {
                self.decode(&mut rams[r2]);
                self.decode(&mut rams[r1]);
                let r1_ram = rams[r1].clone();
                self.xor(&r1_ram, &mut rams[r2]);
            }
            _ => unreachable!(),
        }

        assert_eq!(rams[0].len(), self.size);
        assert_eq!(rams[1].len(), self.size);

        let mut result = Vec::with_capacity(self.size * 2);
        for (a, b) in rams[0].iter().zip(rams[1].iter()) {
            result.push(*a);
            result.push(*b);
        }

        result
    }

    fn fillram(&mut self, ram: &mut Vec<u8>) {
        let mut mode = self.bs.next();
        let size = self.size * 4;
        while ram.len() < size {
            if !mode {
                self.read_rle_chunk(ram);
                mode = true;
            } else {
                self.read_data_chunk(ram, size);
                mode = false;
            }
        }
        assert_eq!(ram.len(), size);
        self.deinterlace_bitgroups(ram);
    }

    fn read_rle_chunk(&mut self, ram: &mut Vec<u8>) {
        let mut i = 0;

        while self.bs.next() {
            i += 1;
        }

        let mut n = TABLE1[i];
        let a = self.bs.read_int(i + 1);
        n += a;

        for _ in 0..n {
            ram.push(0);
        }
    }

    fn read_data_chunk(&mut self, ram: &mut Vec<u8>, size: usize) {
        loop {
            let bitgroup = self.bs.read_int(2);
            if bitgroup == 0 {
                break;
            }
            ram.push(bitgroup as u8);

            if size <= ram.len() {
                break;
            }
        }
    }

    fn decode(&self, ram: &mut [u8]) {
        for x in 0..self.sizex {
            let mut bit = 0;
            for y in 0..self.sizey {
                let i = y * self.sizex + x;

                let mut a = (ram[i] >> 4) & 0xf;
                let mut b = ram[i] & 0xf;

                a = TABLE2[bit as usize][a as usize] as u8;
                bit = a & 1;

                b = TABLE2[bit as usize][b as usize] as u8;
                bit = b & 1;

                ram[i] = (a << 4) | b;
            }
        }
    }

    fn xor(&self, ram1: &[u8], ram2: &mut Vec<u8>) {
        for i in 0..ram2.len() {
            ram2[i] ^= ram1[i];
        }
    }

    fn deinterlace_bitgroups(&self, l: &mut Vec<u8>) {
        let bits = std::mem::replace(l, Vec::with_capacity(l.len()));

        for y in 0..self.sizey {
            for x in 0..self.sizex {
                let mut i = 4 * y * self.sizex + x;
                for _ in 0..4 {
                    l.push(bits[i]);
                    i += self.sizex;
                }
            }
        }

        assert_eq!(l.len(), bits.len());
    }
}

fn bitgroups_to_bytes(l: &mut Vec<u8>) {
    let bits = std::mem::replace(l, Vec::with_capacity(l.len() / 4));

    for i in (0..(bits.len() - 3)).step_by(4) {
        let n = (bits[i] << 6) | (bits[i + 1] << 4) | (bits[i + 2] << 2) | bits[i + 3];
        l.push(n);
    }
}

/// Decompress Pokemon Gen I sprite data
pub fn decompress(input: &[u8]) -> Vec<u8> {
    Decompressor::new(input).decompress()
}

/// Transpose square Pokemon Gen I sprite data
///
/// Can panic if the input is not square, or is larger than 15x15 tiles.
pub fn transpose(input: &[u8]) -> Vec<u8> {
    let mut transposed = vec![0; input.len()];

    let width = match input.len() {
        0x010 => 1,
        0x040 => 2,
        0x090 => 3,
        0x100 => 4,
        0x190 => 5,
        0x240 => 6,
        0x310 => 7,
        0x400 => 8,
        0x490 => 9,
        0x540 => 10,
        0x610 => 11,
        0x700 => 12,
        0x790 => 13,
        0x840 => 14,
        0x910 => 15,
        _ => panic!("input is not a square, or is larger than 15x15 tiles"),
    };

    for i in 0..input.len() {
        let j = (i / 0x10) * width * 0x10;
        let j = (j % input.len()) + 0x10 * (j / input.len()) + (i % 0x10);
        transposed[j] = input[i];
    }

    transposed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fossilaerodactyl() {
        let input = include_bytes!("../fixtures/gen1/fossilaerodactyl.pic");
        let expected = include_bytes!("../fixtures/gen1/fossilaerodactyl.2bpp");

        let actual = transpose(&decompress(input));

        assert_eq!(actual, expected);
    }

    #[test]
    fn fossilkabutops() {
        let input = include_bytes!("../fixtures/gen1/fossilkabutops.pic");
        let expected = include_bytes!("../fixtures/gen1/fossilkabutops.2bpp");

        let actual = transpose(&decompress(input));

        assert_eq!(actual, expected);
    }

    #[test]
    fn ghost() {
        let input = include_bytes!("../fixtures/gen1/ghost.pic");
        let expected = include_bytes!("../fixtures/gen1/ghost.2bpp");

        let actual = transpose(&decompress(input));

        assert_eq!(actual, expected);
    }
}
