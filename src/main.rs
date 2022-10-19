use std::fs::File;
use std::io::BufReader;

mod canvas;
mod coord;
mod graph;

fn main() {
    let mut f = BufReader::new(
        File::open("test.plain")
            .expect("open failed"),
    );
    dbg!(graph::Graph::from_plain(&mut f).unwrap());

    /*let f = canvas::Font::generate(&[
        (' ', canvas::LineArt::from_bits(0b000_000_000_000_000)),
        ('_', canvas::LineArt::from_bits(0b000_000_000_000_111)),
        ('.', canvas::LineArt::from_bits(0b000_000_000_111_000)),
        ('-', canvas::LineArt::from_bits(0b000_000_111_000_000)),
        ('\'', canvas::LineArt::from_bits(0b000_111_000_000_000)),
        ('`', canvas::LineArt::from_bits(0b111_000_000_000_000)),
        ('|', canvas::LineArt::from_bits(0b001_001_001_001_001)),
        ('|', canvas::LineArt::from_bits(0b010_010_010_010_010)),
        ('|', canvas::LineArt::from_bits(0b100_100_100_100_100)),
        ('+', canvas::LineArt::from_bits(0b010_010_111_010_010)),
        ('.', canvas::LineArt::from_bits(0b000_000_111_010_010)),
        ('\'', canvas::LineArt::from_bits(0b010_010_111_000_000)),
        ('\\', canvas::LineArt::from_bits(0b100_110_010_011_001)),
        ('/', canvas::LineArt::from_bits(0b001_011_010_110_100)),
        ('[', canvas::LineArt::from_bits(0b011_010_010_010_011)),
        (']', canvas::LineArt::from_bits(0b110_010_010_010_110)),
        ('(', canvas::LineArt::from_bits(0b001_010_010_010_001)),
        (')', canvas::LineArt::from_bits(0b100_010_010_010_100)),
        ('{', canvas::LineArt::from_bits(0b011_010_110_010_011)),
        ('}', canvas::LineArt::from_bits(0b110_010_011_010_110)),
        ('<', canvas::LineArt::from_bits(0b001_010_100_010_001)),
        ('>', canvas::LineArt::from_bits(0b100_010_001_010_100)),
        ('.', canvas::LineArt::from_bits(0b000_000_000_010_000)),
        (',', canvas::LineArt::from_bits(0b000_000_000_010_100)),
        ('=', canvas::LineArt::from_bits(0b000_111_000_111_000)),
        ('\'', canvas::LineArt::from_bits(0b010_010_000_000_000)),
        ('"', canvas::LineArt::from_bits(0b101_101_000_000_000)),
        ('`', canvas::LineArt::from_bits(0b100_010_000_000_000)),
        ('+', canvas::LineArt::from_bits(0b000_010_111_010_000)),
        ('#', canvas::LineArt::from_bits(0b101_111_101_111_101)),
    ]);

    dbg!(f.serialize());

    //let f = Font::default();
    dbg!(f.translate(LineArt::from_bits(0b111_000_000_000_000)));
    dbg!(f.translate(LineArt::from_bits(0b000_111_000_000_000)));
    dbg!(f.translate(LineArt::from_bits(0b000_000_111_000_000)));
    dbg!(f.translate(LineArt::from_bits(0b000_000_000_111_000)));
    dbg!(f.translate(LineArt::from_bits(0b000_000_000_000_111)));*/

    let mut c = canvas::Canvas::new(200.0);
    c.draw_string(&canvas::InputCoord { x: 100.0, y: 100.0 }, "hello");
    c.draw_rect(
        &canvas::InputCoord { x: 50.0, y: 50.0 },
        &canvas::InputCoord { x: 150.0, y: 150.0 },
    );
    c.draw_line(
        &canvas::InputCoord { x: 50.0, y: 50.0 },
        &canvas::InputCoord { x: 150.0, y: 150.0 },
    );
    c.draw_line(
        &canvas::InputCoord { x: 50.0, y: 110.0 },
        &canvas::InputCoord { x: 150.0, y: 90.0 },
    );
    println!("{c}");
}
