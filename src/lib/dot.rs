use std::collections::HashMap;
use std::io::BufRead;
use utf8_chars::BufReadCharsExt;

pub trait Scalable {
    fn scale(&mut self, scale: f64);
}

#[derive(Clone, Debug)]
pub struct Graph {
    pub width: f64,
    pub height: f64,
    pub nodes: HashMap<String, Node>,
    pub edges: Vec<Edge>,
}

impl Graph {
    /// "Parses" a string word, throwing a reasonable error message on failure.
    fn parse_string(words: &[String], line_no: usize, word_idx: usize) -> Result<String, String> {
        if let Some(word) = words.get(word_idx) {
            Ok(word.to_string())
        } else {
            Err(format!(
                "argument index {word_idx} out of range on line {line_no}"
            ))
        }
    }

    /// Parses a floating-point word, throwing a reasonable error message on
    /// failure.
    fn parse_float(words: &[String], line_no: usize, word_idx: usize) -> Result<f64, String> {
        if let Some(word) = words.get(word_idx) {
            word.parse().map_err(|e| format!("failed to parse argument {word_idx} on line {line_no} as float ('{word}'): {e}"))
        } else {
            Err(format!(
                "argument index {word_idx} out of range on line {line_no}"
            ))
        }
    }

    /// Parses a list length word, throwing a reasonable error message on
    /// failure.
    fn parse_usize(words: &[String], line_no: usize, word_idx: usize) -> Result<usize, String> {
        if let Some(word) = words.get(word_idx) {
            word.parse().map_err(|e| format!("failed to parse argument {word_idx} on line {line_no} as integer ('{word}'): {e}"))
        } else {
            Err(format!(
                "argument index {word_idx} out of range on line {line_no}"
            ))
        }
    }

    /// Parses a floating-point word pair as a coordinate, throwing a
    /// reasonable error message on failure.
    fn parse_coord(words: &[String], line_no: usize, word_idx: usize) -> Result<Coord, String> {
        Ok(Coord {
            x: Graph::parse_float(words, line_no, word_idx)?,
            y: Graph::parse_float(words, line_no, word_idx + 1)?,
        })
    }

    /// Parses the dot plain text output format into a graph.
    pub fn from_plain<T: BufRead>(input: &mut T) -> Result<Graph, String> {
        // Create an empty graph structure for us to populate.
        let mut graph = Graph {
            width: 0.0,
            height: 0.0,
            nodes: HashMap::new(),
            edges: Vec::new(),
        };
        let mut scale = 1.0;

        // Parse dot's plain text output format.
        let mut words = vec![String::new()];
        let mut in_string = false;
        let mut escaping = false;
        let mut line_no = 1;
        for char in input.chars() {
            let char = char.map_err(|e| format!("read failed: {e:?}"))?;
            if char == '\n' || char == '\r' {
                if words.last().unwrap().is_empty() {
                    words.pop();
                }
                match words.first().map(|x| &x[..]) {
                    Some("graph") => {
                        if words.len() != 4 {
                            return Err(format!(
                                "expected 3 arguments for graph statement on line {line_no}"
                            ));
                        }
                        scale = Graph::parse_float(&words, line_no, 1)?;
                        graph.width = Graph::parse_float(&words, line_no, 2)?;
                        graph.height = Graph::parse_float(&words, line_no, 3)?;
                    }
                    Some("node") => {
                        if words.len() != 11 {
                            return Err(format!(
                                "expected 10 arguments for node statement on line {line_no}"
                            ));
                        }
                        let name = Graph::parse_string(&words, line_no, 1)?;
                        let node = Node {
                            name: name.clone(),
                            coord: Graph::parse_coord(&words, line_no, 2)?,
                            size: Graph::parse_coord(&words, line_no, 4)?,
                            label: Graph::parse_string(&words, line_no, 6)?,
                            style: Graph::parse_string(&words, line_no, 7)?,
                            shape: Graph::parse_string(&words, line_no, 8)?,
                            color: Graph::parse_string(&words, line_no, 9)?,
                            fillcolor: Graph::parse_string(&words, line_no, 10)?,
                        };
                        if graph.nodes.insert(name.clone(), node).is_some() {
                            return Err(format!("duplicate node name {name} on line {line_no}"));
                        }
                    }
                    Some("edge") => {
                        let tail = Graph::parse_string(&words, line_no, 1)?;
                        if graph.nodes.get(&tail).is_none() {
                            return Err(format!(
                                "unknown node {tail} used for edge on line {line_no}"
                            ));
                        }
                        let head = Graph::parse_string(&words, line_no, 2)?;
                        if graph.nodes.get(&head).is_none() {
                            return Err(format!(
                                "unknown node {head} used for edge on line {line_no}"
                            ));
                        }
                        let num_cpts = Graph::parse_usize(&words, line_no, 3)?;
                        let cpts: Vec<Coord> = (0..num_cpts)
                            .into_iter()
                            .map(|i| Graph::parse_coord(&words, line_no, 4 + i * 2))
                            .collect::<Result<_, _>>()?;
                        let label = if words.len() == 9 + num_cpts * 2 {
                            Some(Label {
                                text: Graph::parse_string(&words, line_no, 4 + num_cpts * 2)?,
                                coord: Graph::parse_coord(&words, line_no, 5 + num_cpts * 2)?,
                            })
                        } else if words.len() == 6 + num_cpts * 2 {
                            None
                        } else {
                            return Err(format!("unexpected number of arguments for edge statement on line {line_no}"));
                        };
                        graph.edges.push(Edge {
                            tail,
                            head,
                            cpts,
                            label,
                            style: Graph::parse_string(&words, line_no, words.len() - 2)?,
                            color: Graph::parse_string(&words, line_no, words.len() - 1)?,
                        });
                    }
                    Some("stop") => {
                        if words.len() != 1 {
                            return Err(format!(
                                "expected zero argument for stop statement on line {line_no}"
                            ));
                        }
                    }
                    Some(unknown) => {
                        return Err(format!("unrecognized command {unknown} on line {line_no}"));
                    }
                    None => (),
                }
                words.clear();
                words.push(String::new());
                in_string = false;
                escaping = false;
                line_no += 1;
            } else if escaping {
                words.last_mut().unwrap().push(match char {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    c => c,
                });
                escaping = false;
            } else if char == '"' {
                in_string = !in_string;
            } else if char == ' ' && !in_string {
                if !words.last().unwrap().is_empty() {
                    words.push(String::new())
                }
            } else if char == '\\' && in_string {
                escaping = true;
            } else {
                words.last_mut().unwrap().push(char);
            }
        }

        // Apply the scale factor so we don't need to worry about it anymore.
        graph.scale(scale);

        Ok(graph)
    }
}

impl Scalable for Graph {
    fn scale(&mut self, scale: f64) {
        self.width *= scale;
        self.height *= scale;
        for (_, node) in self.nodes.iter_mut() {
            node.scale(scale);
        }
        for edge in self.edges.iter_mut() {
            edge.scale(scale)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub name: String,
    pub coord: Coord,
    pub size: Coord,
    pub label: String,
    pub style: String,
    pub shape: String,
    pub color: String,
    pub fillcolor: String,
}

impl Scalable for Node {
    fn scale(&mut self, scale: f64) {
        self.coord.scale(scale);
        self.size.scale(scale);
    }
}

#[derive(Clone, Debug)]
pub struct Edge {
    pub tail: String,
    pub head: String,
    pub cpts: Vec<Coord>,
    pub label: Option<Label>,
    pub style: String,
    pub color: String,
}

impl Scalable for Edge {
    fn scale(&mut self, scale: f64) {
        for cpt in self.cpts.iter_mut() {
            cpt.scale(scale);
        }
        if let Some(label) = &mut self.label {
            label.scale(scale)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Label {
    pub text: String,
    pub coord: Coord,
}

impl Scalable for Label {
    fn scale(&mut self, scale: f64) {
        self.coord.scale(scale);
    }
}

pub type Coord = vector2d::Vector2D<f64>;

impl Scalable for Coord {
    fn scale(&mut self, scale: f64) {
        self.x *= scale;
        self.y *= scale;
    }
}
