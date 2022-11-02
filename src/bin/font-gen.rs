//! Fonts are used to map 3x5-pixel bitmaps to a best-matching character by
//! some heuristic metric. They are represented as a 32k character lookup
//! table; one character for each possible bitmap. This binary generates such
//! a lookup table. It does this based on a list of reference characters with
//! a specified bitmap, choosing the character based on the most similar
//! reference bitmap. The similarity metric is pretty slow, hence the need for
//! a lookup table and offline generation of that table.
//!
//! Feel free to change the font by changing the reference table! The bitmap
//! is specified using binary in left to right, top to bottom order.

use std::io::Write;

use dot_txt::canvas;

fn main() {
    eprintln!();
    let f = canvas::BitmapFont::generate(
        &[
            (' ', canvas::BitmapChar::from_bits(0b000_000_000_000_000)),
            ('_', canvas::BitmapChar::from_bits(0b000_000_000_000_111)),
            ('.', canvas::BitmapChar::from_bits(0b000_000_000_111_000)),
            ('-', canvas::BitmapChar::from_bits(0b000_000_111_000_000)),
            ('\'', canvas::BitmapChar::from_bits(0b000_111_000_000_000)),
            ('`', canvas::BitmapChar::from_bits(0b111_000_000_000_000)),
            ('|', canvas::BitmapChar::from_bits(0b001_001_001_001_001)),
            ('|', canvas::BitmapChar::from_bits(0b010_010_010_010_010)),
            ('|', canvas::BitmapChar::from_bits(0b100_100_100_100_100)),
            ('+', canvas::BitmapChar::from_bits(0b010_010_111_010_010)),
            ('.', canvas::BitmapChar::from_bits(0b000_000_100_100_100)),
            ('.', canvas::BitmapChar::from_bits(0b000_000_010_010_010)),
            ('.', canvas::BitmapChar::from_bits(0b000_000_001_001_001)),
            ('\'', canvas::BitmapChar::from_bits(0b100_100_100_000_000)),
            ('\'', canvas::BitmapChar::from_bits(0b010_010_010_000_000)),
            ('\'', canvas::BitmapChar::from_bits(0b001_001_001_000_000)),
            ('\\', canvas::BitmapChar::from_bits(0b100_110_010_011_001)),
            ('/', canvas::BitmapChar::from_bits(0b001_011_010_110_100)),
            ('[', canvas::BitmapChar::from_bits(0b011_010_010_010_011)),
            (']', canvas::BitmapChar::from_bits(0b110_010_010_010_110)),
            ('(', canvas::BitmapChar::from_bits(0b001_010_010_010_001)),
            (')', canvas::BitmapChar::from_bits(0b100_010_010_010_100)),
            ('{', canvas::BitmapChar::from_bits(0b011_010_110_010_011)),
            ('}', canvas::BitmapChar::from_bits(0b110_010_011_010_110)),
            ('<', canvas::BitmapChar::from_bits(0b001_010_100_010_001)),
            ('>', canvas::BitmapChar::from_bits(0b100_010_001_010_100)),
            ('.', canvas::BitmapChar::from_bits(0b000_000_000_010_000)),
            (',', canvas::BitmapChar::from_bits(0b000_000_000_010_100)),
            ('=', canvas::BitmapChar::from_bits(0b000_111_000_111_000)),
            ('\'', canvas::BitmapChar::from_bits(0b010_010_000_000_000)),
            ('"', canvas::BitmapChar::from_bits(0b101_101_000_000_000)),
            ('`', canvas::BitmapChar::from_bits(0b100_010_000_000_000)),
            ('+', canvas::BitmapChar::from_bits(0b000_010_111_010_000)),
            ('#', canvas::BitmapChar::from_bits(0b101_111_101_111_101)),
        ],
        |progress| {
            eprintln!("\r\x1B[A\x1B[KGenerating... {:.01}%", progress * 100f32);
        },
    );
    let mut file = std::fs::File::create("src/lib/font.txt").expect("failed to open output file");
    file.write_all(f.serialize().as_bytes())
        .expect("failed to write to output file");
    eprintln!("\r\x1B[A\x1B[KGenerating... done");
}
