/// Decompress Pokemon Gen III data in the GBA LZ77 (LZ10) format.
///
/// The input starts at the LZ10 header. Bytes after the first compressed
/// stream are ignored, so the input can be a slice from the asset's ROM offset.
/// For sprite data, the output is the raw GBA 4bpp tile data.
///
/// # Panics
///
/// Panics if the header is invalid, the stream is truncated, or a back-reference
/// falls outside the decoded prefix or the declared output size.
pub fn decompress(input: &[u8]) -> Vec<u8> {
    assert!(input.len() >= 4, "truncated LZ10 header");
    assert_eq!(input[0], 0x10, "invalid LZ10 compression type");

    let output_len = u32::from_le_bytes([input[1], input[2], input[3], 0]) as usize;
    let mut output = Vec::with_capacity(output_len);
    let mut pos = 4;

    while output.len() < output_len {
        let flags = input[pos];
        pos += 1;

        for bit in (0..8).rev() {
            if flags & (1 << bit) == 0 {
                output.push(input[pos]);
                pos += 1;
            } else {
                let pair = u16::from_be_bytes([input[pos], input[pos + 1]]);
                pos += 2;

                let length = usize::from(pair >> 12) + 3;
                let distance = usize::from(pair & 0x0fff) + 1;
                assert!(distance <= output.len(), "invalid LZ10 back-reference");
                assert!(
                    length <= output_len - output.len(),
                    "LZ10 back-reference exceeds declared output size"
                );

                // A match can overlap the bytes it produces.
                for _ in 0..length {
                    output.push(output[output.len() - distance]);
                }
            }

            if output.len() == output_len {
                break;
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATIC_INPUT: &[u8] = include_bytes!("../fixtures/gen3/pikachu-front.4bpp.lz");
    const STATIC_TILES: &[u8] = include_bytes!("../fixtures/gen3/pikachu-front.4bpp");
    const ANIMATED_INPUT: &[u8] = include_bytes!("../fixtures/gen3/pikachu-anim-front.4bpp.lz");
    const ANIMATED_TILES: &[u8] = include_bytes!("../fixtures/gen3/pikachu-anim-front.4bpp");

    #[test]
    fn pikachu_front() {
        assert_eq!(decompress(STATIC_INPUT), STATIC_TILES);
    }

    #[test]
    fn pikachu_animated_front() {
        assert_eq!(decompress(ANIMATED_INPUT), ANIMATED_TILES);
    }

    #[test]
    fn rom_slice_with_trailing_bytes() {
        let mut rom = vec![0xff; 16];
        let offset = rom.len();
        rom.extend_from_slice(ANIMATED_INPUT);
        rom.extend_from_slice(STATIC_INPUT);

        assert_eq!(decompress(&rom[offset..]), ANIMATED_TILES);
    }
}
