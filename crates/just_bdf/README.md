# `just_bdf`

[Glyph Bitmap Distribution Format](https://en.wikipedia.org/wiki/Glyph_Bitmap_Distribution_Format) parser.

## Usage

```rust
use std::{fs::File, io::Read};

let mut unparsed_font = String::new();
File::open("./path/to/font.bdf")
    .expect("Could not open font file")
    .read_to_string(&mut unparsed_font)
    .expect("Could not read font file");
let font = just_bdf::parse(&unparsed_font)
    .expect("Could not parse font file");
```
