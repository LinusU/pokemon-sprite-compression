#[rustfmt::skip]
const INV_XOR_TABLE: [[u8; 16]; 4] = [
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

    fn read_compress_int(&mut self) -> usize {
        let mut n = 1;

        while self.next() {
            n += 1;
        }

        ((1 << n) | self.read_int(n)) - 1
    }
}

struct Decompressor<'a> {
    data: BitStream<'a>,
    width: usize,
    height: usize,
}

impl<'a> Decompressor<'a> {
    fn new(data: &'a [u8]) -> Self {
        let mut data = BitStream::new(data);

        let width = data.read_int(4);
        let height = data.read_int(4);

        Self {
            data,
            width,
            height,
        }
    }

    fn decompress(&mut self) -> Vec<u8> {
        let order_reversed = self.data.next();

        let mut ram0 = self.fillram();

        let mode = if self.data.next() {
            if self.data.next() {
                2
            } else {
                1
            }
        } else {
            0
        };

        let mut ram1 = self.fillram();

        match mode {
            0 => {
                self.decode(&mut ram0);
                self.decode(&mut ram1);
            }
            1 => {
                self.decode(&mut ram0);
                self.xor(&ram0, &mut ram1);
            }
            2 => {
                self.decode(&mut ram1);
                self.decode(&mut ram0);
                self.xor(&ram0, &mut ram1);
            }
            _ => unreachable!(),
        }

        let mut result = Vec::with_capacity(ram0.len() + ram1.len());

        for (a, b) in ram0.iter().zip(ram1.iter()) {
            if order_reversed {
                result.push(*b);
                result.push(*a);
            } else {
                result.push(*a);
                result.push(*b);
            }
        }

        result
    }

    fn fillram(&mut self) -> Vec<u8> {
        let plane_width = self.width * TILESIZE;
        let size = plane_width * self.height;

        let mut z = if self.data.next() {
            0
        } else {
            self.data.read_compress_int()
        };

        let mut interlaced = Vec::with_capacity(size);

        while interlaced.len() < size {
            let mut byte: u8 = 0;

            for shift in [6, 4, 2, 0] {
                if z > 0 {
                    z -= 1;
                    continue;
                }

                let bitgroup = self.data.read_int(2) as u8;

                if bitgroup == 0 {
                    z = self.data.read_compress_int() - 1;
                    continue;
                }

                byte |= bitgroup << shift;
            }

            interlaced.push(byte);
        }

        let mut deinterlaced = Vec::with_capacity(size);

        for y in 0..self.height {
            for x in 0..plane_width {
                let bit_shift = 6 - ((x % 4) * 2);
                let byte_index = (y * plane_width) + (x / 4);

                deinterlaced.push(
                    ((interlaced[byte_index] >> bit_shift) & 0b11) << 6
                        | ((interlaced[byte_index + self.width * 2] >> bit_shift) & 0b11) << 4
                        | ((interlaced[byte_index + self.width * 4] >> bit_shift) & 0b11) << 2
                        | ((interlaced[byte_index + self.width * 6] >> bit_shift) & 0b11),
                );
            }
        }

        deinterlaced
    }

    fn decode(&self, ram: &mut [u8]) {
        let plane_width = self.width * TILESIZE;

        for x in 0..plane_width {
            let mut bit = 0;
            for y in 0..self.height {
                let i = y * plane_width + x;

                let mut a = (ram[i] >> 4) & 0xf;
                let mut b = ram[i] & 0xf;

                a = INV_XOR_TABLE[bit as usize][a as usize];
                bit = a & 1;

                b = INV_XOR_TABLE[bit as usize][b as usize];
                bit = b & 1;

                ram[i] = (a << 4) | b;
            }
        }
    }

    fn xor(&self, ram0: &[u8], ram1: &mut [u8]) {
        for i in 0..ram1.len() {
            ram1[i] ^= ram0[i];
        }
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
