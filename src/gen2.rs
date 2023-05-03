#[rustfmt::skip]
const REVERSE_LOOKUP: [u8; 256] = [
    0,  128, 64, 192, 32, 160,  96, 224, 16, 144, 80, 208, 48, 176, 112, 240,
    8,  136, 72, 200, 40, 168, 104, 232, 24, 152, 88, 216, 56, 184, 120, 248,
    4,  132, 68, 196, 36, 164, 100, 228, 20, 148, 84, 212, 52, 180, 116, 244,
    12, 140, 76, 204, 44, 172, 108, 236, 28, 156, 92, 220, 60, 188, 124, 252,
    2,  130, 66, 194, 34, 162,  98, 226, 18, 146, 82, 210, 50, 178, 114, 242,
    10, 138, 74, 202, 42, 170, 106, 234, 26, 154, 90, 218, 58, 186, 122, 250,
    6,  134, 70, 198, 38, 166, 102, 230, 22, 150, 86, 214, 54, 182, 118, 246,
    14, 142, 78, 206, 46, 174, 110, 238, 30, 158, 94, 222, 62, 190, 126, 254,
    1,  129, 65, 193, 33, 161,  97, 225, 17, 145, 81, 209, 49, 177, 113, 241,
    9,  137, 73, 201, 41, 169, 105, 233, 25, 153, 89, 217, 57, 185, 121, 249,
    5,  133, 69, 197, 37, 165, 101, 229, 21, 149, 85, 213, 53, 181, 117, 245,
    13, 141, 77, 205, 45, 173, 109, 237, 29, 157, 93, 221, 61, 189, 125, 253,
    3,  131, 67, 195, 35, 163,  99, 227, 19, 147, 83, 211, 51, 179, 115, 243,
    11, 139, 75, 203, 43, 171, 107, 235, 27, 155, 91, 219, 59, 187, 123, 251,
    7,  135, 71, 199, 39, 167, 103, 231, 23, 151, 87, 215, 55, 183, 119, 247,
    15, 143, 79, 207, 47, 175, 111, 239, 31, 159, 95, 223, 63, 191, 127, 255
];

/// Decompress Pokemon Gen II sprite data
pub fn decompress(input: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();

    let mut pos = 0;
    loop {
        if input[pos] == 0xff {
            break;
        }

        let mut cmd = input[pos] >> 5;
        let mut count = (input[pos] & 0b00011111) as usize;

        pos += 1;

        // Long command
        if cmd == 7 {
            cmd = (count >> 2) as u8;
            count = ((count & 3) << 8) | (input[pos] as usize);
            pos += 1;
            assert_ne!(cmd, 7);
        }

        count += 1;

        match cmd {
            // Read (x + 1) following amount of bytes from the ROM and print to the graphics output.
            0 => {
                result.extend_from_slice(&input[pos..(pos + count)]);
                pos += count;
            }

            // Read one byte from the ROM, call that value y. Print value (y), (x+1) number of times to the graphics output.
            1 => {
                for _ in 0..count {
                    result.push(input[pos]);
                }

                pos += 1;
            }

            // Read two bytes from the ROM, call those values (y-z).
            // Print y and z alternately (x+1) number of times total to the graphics output.
            // Example: If x = 5, print yzyzy.
            2 => {
                for i in 0..count {
                    result.push(input[pos + (i % 2)]);
                }

                pos += 2;
            }

            // Print (x+1) bytes with value zero to the graphics output.
            3 => {
                result.resize(result.len() + count, 0);
            }

            // Read a byte from the rom, call that value A.
            // If the high bit of A (Bitwise AND with 128 - 0x80, binary 10000000) is clear:
            // Read the next byte from the rom, call that value N.
            // Copy (x+1) consecutive bytes from the graphics buffer to the graphics output, starting at (A*0x100) + (N+1).
            //
            // If the high bit of A is set:
            // (AND A with 127(0x7F) to clear the high bit)
            // Copy (x+1) consecutive bytes from the graphics buffer to the graphics output, starting A bytes back from the end of the buffer.
            4 => {
                let start = if input[pos] & 0x80 == 0 {
                    let a = ((input[pos] as usize) << 8) | (input[pos + 1] as usize);
                    pos += 2;
                    a
                } else {
                    let a = (input[pos] & 0x7f) as usize;
                    pos += 1;
                    result.len() - a - 1
                };

                for i in 0..count {
                    result.push(result[start + i]);
                }
            }

            // Read a byte from the rom, call that value A.
            // If the high bit of A is clear:
            // Read the next byte from the rom, call that value N.
            // Copy (x+1) consecutive bytes from the graphics buffer to the graphics output, starting at (A*0x100) + (N+1), REVERSING the bit order.
            //  Example: Byte k in graphics buffer = 0xD7 (11010111 binary).
            //  New byte to be printed to graphics output = 0xEB(11101011 binary).
            //
            // If the high bit of A is set:
            // (clear the high bit of A)
            // Copy (x+1) consecutive bytes from the graphics buffer to the graphics output, starting A bytes back from the end of the buffer, REVERSING the bit order.
            5 => {
                let start = if input[pos] & 0x80 == 0 {
                    let a = ((input[pos] as usize) << 8) | (input[pos + 1] as usize);
                    pos += 2;
                    a
                } else {
                    let a = (input[pos] & 0x7f) as usize;
                    pos += 1;
                    result.len() - a - 1
                };

                for i in start..(start + count) {
                    result.push(REVERSE_LOOKUP[result[i] as usize]);
                }
            }

            // Read a byte from the rom, call that value A.
            // If the high bit of A is clear:
            // Read the next byte from the rom, call that value N.
            // Copy (x+1) reverse consecutive bytes from the graphics buffer to the graphics output, starting at (A*0x100) + (N+1).
            // "reverse consecutive" meaning starting from the above point in the graphics buffer, read a byte from buffer and print to the end of the output, then read the previous byte and print to the end of the output, etc.
            //
            // If the high bit of A is set:
            // (clear the high bit of A)
            // Copy (x+1) reverse consecutive bytes from the graphics buffer to the graphics output, starting A bytes back from the end of the buffer.
            6 => {
                let start = if input[pos] & 0x80 == 0 {
                    let a = ((input[pos] as usize) << 8) | (input[pos + 1] as usize);
                    pos += 2;
                    a
                } else {
                    let a = (input[pos] & 0x7f) as usize;
                    pos += 1;
                    result.len() - a - 1
                };

                for i in 0..count {
                    result.push(result[start - i]);
                }
            }

            _ => unreachable!(),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bellsprout() {
        let input = include_bytes!("../fixtures/bellsprout.2bpp.lz");
        let expected = include_bytes!("../fixtures/bellsprout.2bpp");

        let actual = decompress(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn dugtrio() {
        let input = include_bytes!("../fixtures/dugtrio.2bpp.lz");
        let expected = include_bytes!("../fixtures/dugtrio.2bpp");

        let actual = decompress(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn pikachu() {
        let input = include_bytes!("../fixtures/pikachu.2bpp.lz");
        let expected = include_bytes!("../fixtures/pikachu.2bpp");

        let actual = decompress(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn steelix() {
        let input = include_bytes!("../fixtures/steelix.2bpp.lz");
        let expected = include_bytes!("../fixtures/steelix.2bpp");

        let actual = decompress(input);

        assert_eq!(actual, expected);
    }
}
