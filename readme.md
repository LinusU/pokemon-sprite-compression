# Pokemon Sprite Compression

A library for dealing with compressed Pokemon sprites.

Implementation status:

- [x] Gen 1
    - [x] Decompression
    - [ ] Compression
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
const rom = std::fs::read("pokeyellow.gbc").unwrap();

// Aerodactyl fossil sprite
let sprite = pokemon_sprite_compression::gen1::decompress(&rom[0x0367a1..]);
```

```rust
const rom = std::fs::read("pokecrystal.gbc").unwrap();

// Pikachu back sprite
let sprite = pokemon_sprite_compression::gen2::decompress(&rom[0x156ea1..]);
```

## Acknowledgements

Huge thanks to the [pret team](https://github.com/orgs/pret/people) and [Andrew Ekstedt](https://github.com/magical) for their work on [gen I decompression](https://github.com/pret/pokered/blob/1b9540cc49f76626445d2cb5744be1e81c709ca8/tools/pic.py).

Huge thanks to [wgjordan on Hacker News](https://news.ycombinator.com/user?id=wgjordan) for [reverse engineering and documenting the gen II compression algorithm](https://www.romhacking.net/utilities/59/). I've included a copy of the documentation in this repository for convenience, as the file `doc/gen2.txt`.
