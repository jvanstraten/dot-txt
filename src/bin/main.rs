use dot_txt::canvas;
use dot_txt::dot;
use dot_txt::dot::Coord;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let mut file = BufReader::new(File::open("test.plain").expect("open failed"));
    let graph = dot::Graph::from_plain(&mut file).expect("parse failed");
    drop(file);

    let mut c = canvas::Canvas::new(200.0, Coord::new(50.0, 50.0));
    for (_, node) in graph.nodes.iter() {
        c.draw_rect(node.coord - node.size / 2.0, node.coord + node.size / 2.0);
    }
    for edge in graph.edges.iter() {
        let mut iter = edge.cpts.iter();
        if let Some(mut a) = iter.next() {
            for b in iter {
                c.draw_line(*a, *b);
                a = b;
            }
        }
    }

    /*c.draw_string(&Coord { x: 100.0, y: 100.0 }, "hello");
    c.draw_rect(
        &Coord { x: 50.0, y: 50.0 },
        &Coord { x: 150.0, y: 150.0 },
    );
    c.draw_line(
        &Coord { x: 50.0, y: 50.0 },
        &Coord { x: 150.0, y: 150.0 },
    );
    c.draw_line(
        &Coord { x: 50.0, y: 110.0 },
        &Coord { x: 150.0, y: 90.0 },
    );*/
    println!("{c}");
    println!("{c:#}");
}
