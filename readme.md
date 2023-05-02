# Pokemon Sprite Compression

A library for dealing with compressed Pokemon sprites.

Implementation status:

- [ ] Gen 1
- [x] Gen 2
    - [x] Decompression
    - [ ] Compression
- [ ] Future generations

## Installation

```sh
cargo add pokemon-sprite-compression
```

## Usage

```rust
const rom = std::fs::read("pokecrystal.gbc").unwrap();

// Pikachu back sprite
let sprite = pokemon_sprite_compression::gen2::decompress(&rom[0x156ea1..]);
```

## Acknowledgements

Huge thanks to [wgjordan on Hacker News](https://news.ycombinator.com/user?id=wgjordan) for [reverse engineering and documenting the compression algorithm](https://www.romhacking.net/utilities/59/). I've included a copy of the documentation in this repository for convenience, as the file `doc/gen2.txt`.
