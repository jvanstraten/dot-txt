/// Represents a 3x5 pixel bitmap for a character position. The best option
/// will be chosen. In LSB to MSB-1 order, the pixels are ordered
/// left-to-right, top-to-bottom.
#[derive(Clone, Copy, Default)]
pub struct BitmapChar(u16);

impl BitmapChar {
    /// Creates a pixels object from bits. Bit order is swapped, so the binary
    /// representation is a bit easier to read. For example,
    /// 0b000_000_000_000_111 is an '_', 0b011_010_010_010_011 is a '[', and
    /// so on.
    pub fn from_bits(mut bits: u16) -> BitmapChar {
        bits <<= 1;
        bits = (bits >> 8) | (bits << 8);
        bits = ((bits >> 4) & 0x0F0F) | ((bits << 4) & 0xF0F0);
        bits = ((bits >> 2) & 0x3333) | ((bits << 2) & 0xCCCC);
        bits = ((bits >> 1) & 0x5555) | ((bits << 1) & 0xAAAA);
        BitmapChar(bits)
    }

    /// Read the pixel at the given coordinate, where 0,0 is top-left and 2,4 is
    /// bottom-right. Out-of-range accesses yield false.
    fn peek(&self, x: i8, y: i8) -> bool {
        x >= 0 && y >= 0 && x < 3 && y < 5 && (self.0 & (1u16 << (x + y * 3))) != 0
    }

    /// Set the pixel at the given coordinate, where 0,0 is top-left and 2,4 is
    /// bottom-right. Out-of-range accesses are ignored.
    fn poke(&mut self, x: i8, y: i8, value: bool) {
        if x >= 0 && y >= 0 && x < 3 && y < 5 {
            let mask = 1u16 << (x + y * 3);
            if value {
                self.0 |= mask;
            } else {
                self.0 &= !mask;
            }
        }
    }

    /// Internal helper for similarity().
    fn similarity_asym(self, other: BitmapChar) -> f64 {
        let mut sim = 0.0;
        for x in 0..=2 {
            for y in 0..=4 {
                if self.peek(x, y) {
                    let mut pixel_min = 2.0f64;
                    for sx in -1..=1 {
                        for sy in -1..=1 {
                            if other.peek(x + sx, y + sy) {
                                pixel_min = pixel_min.min(((sx * sx + sy * sy) as f64).sqrt());
                            }
                        }
                    }
                    sim += pixel_min;
                }
            }
        }
        sim
    }

    /// Scores how visually close the two pixel maps are to one another. Zero
    /// means equal, increasingly higher values mean increasingly dissimilar.
    /// Note that this is a throw-stuff-at-the-wall-and-see-what-sticks
    /// implementation, and it is SLOW. It should only be used to build static
    /// lookup tables.
    fn similarity(self, other: BitmapChar) -> f64 {
        self.similarity_asym(other) + other.similarity_asym(self)
    }
}

/// A 3x5 pixel to character lookup table for box drawing.
pub struct BitmapFont {
    data: [char; 32768],
}

impl BitmapFont {
    /// Generates a font. Each character in the lookup table is selected from
    /// the given character set based on a similarity heuristic.
    pub fn generate<F: FnMut(f32)>(charset: &[(char, BitmapChar)], mut progress: F) -> BitmapFont {
        let mut font = BitmapFont {
            data: ['\0'; 32768],
        };
        for index in 0b000_000_000_000_000..=0b111_111_111_111_111 {
            let target = BitmapChar(index);
            let mut best_sim = f64::INFINITY;
            let mut best_char = '?';
            for (c, actual) in charset.iter() {
                let sim = target.similarity(*actual);
                if sim < best_sim {
                    best_sim = sim;
                    best_char = *c;
                }
            }
            font.data[index as usize] = best_char;
            if index % 100 == 0 {
                progress(index as f32 / 0b111_111_111_111_111 as f32);
            }
        }
        font
    }

    /// Deserializes a font from a 32k-character string.
    pub fn deserialize(data: &str) -> BitmapFont {
        assert!(data.len() == 32768);
        let mut font = BitmapFont {
            data: ['\0'; 32768],
        };
        for (i, c) in data.chars().enumerate() {
            font.data[i] = c
        }
        font
    }

    /// Serializes a font into a 32k-character string.
    pub fn serialize(&self) -> String {
        self.data.iter().cloned().collect()
    }

    /// Translates a 3x5 bitmap to its best character representation.
    pub fn translate(&self, pixels: BitmapChar) -> char {
        self.data[pixels.0 as usize]
    }
}

impl Default for BitmapFont {
    fn default() -> Self {
        BitmapFont::deserialize(include_str!("font.txt"))
    }
}

/// A character in the canvas. Either a 3x5 pixel map or a textual character.
/// Textual characters always take precedence over line art.
#[derive(Clone, Copy)]
enum Character {
    Bitmap(BitmapChar),
    Text(char),
}

impl Default for Character {
    fn default() -> Self {
        Character::Bitmap(BitmapChar::default())
    }
}

/// ASCII art canvas.
///
/// The mapping from float coordinates to character coordinates is as follows:
///
///  - Column = floor(x * scale.x / 3)
///  - Row = floor(y * scale.y / 5)
///
/// Each character position can also be treated as a 3x5-pixel bitmap, hence
/// the divisions there. The pixel coordinates are thus mapped as follows:
///
///  - Column = floor(x * scale.x)
///  - Row = floor(y * scale.y)
///
/// Writing a normal (text) character to a character position overrides any
/// bitmap information for that position. If, in the end, only a bitmap is left
/// for some position, a suitable character is chosen based on a BitmapFont
/// lookup table.
#[derive(Clone)]
pub struct Canvas {
    /// Characters arranged in lines of lenth width, not including newlines and
    /// the likes. Initialized with spaces.
    data: Vec<Character>,

    /// Width of the data buffer.
    width: usize,

    /// Scaling factor (x and y independently). For the default unit scale, a
    /// character is 3x5 coordinate units in size.
    scale: InputCoord,

    /// When labels are too long to fit in a text box, "[<num>]" will be
    /// written instead, where num is one plus the index in this vector.
    footnotes: Vec<String>,
}

impl Canvas {
    /// Creates a new canvas with the specified width.
    pub fn new(width: f64, scale: InputCoord) -> Canvas {
        Canvas {
            data: vec![],
            width: ((width / 3.0) as usize) + 1,
            scale,
            footnotes: vec![],
        }
    }

    /// Returns the index in data for a given character coordinate.
    fn data_index(&self, index: CharCoord) -> Option<usize> {
        if index.x >= self.width {
            None
        } else {
            Some(index.x + self.width * index.y)
        }
    }

    /// Returns the character at the given character coordinate.
    fn get_character(&self, index: CharCoord) -> Character {
        self.data_index(index)
            .map(|i| self.data.get(i))
            .flatten()
            .cloned()
            .unwrap_or(Character::default())
    }

    /// Returns a mutable reference to the character at the given character
    /// coordinate, if it exists.
    fn get_character_mut(&mut self, index: CharCoord) -> Option<&mut Character> {
        if let Some(index) = self.data_index(index) {
            if index >= self.data.len() {
                self.data.resize(index + 1, Character::default());
            }
            Some(&mut self.data[index])
        } else {
            None
        }
    }

    /// Translates a floating-point coordinate to a character coordinate and a
    /// sub-character coordinate on a 3x5 grid per character.
    fn translate_in_to_char(&self, coord: InputCoord) -> Option<(CharCoord, i8, i8)> {
        self.translate_pix_to_char(self.translate_in_to_pix(coord))
    }

    /// Translates a floating-point coordinate to a character coordinate and a
    /// sub-character coordinate on a 3x5 grid per character.
    fn translate_pix_to_char(&self, coord: PixelCoord) -> Option<(CharCoord, i8, i8)> {
        if coord.x < 0 || coord.y < 0 {
            return None;
        }
        let cx = (coord.x / 3) as usize;
        let cy = (coord.y / 5) as usize;
        let px = (coord.x % 3) as i8;
        let py = (coord.y % 5) as i8;
        Some((CharCoord { x: cx, y: cy }, px, py))
    }

    /// Translates a floating-point coordinate to a pixel coordinate.
    fn translate_in_to_pix(&self, coord: InputCoord) -> PixelCoord {
        PixelCoord {
            x: (coord.x * self.scale.x) as i64,
            y: (coord.y * self.scale.y) as i64,
        }
    }

    /// Writes a text character to the given character coordinate.
    fn set_character(&mut self, index: CharCoord, character: char) {
        self.get_character_mut(index)
            .map(|x| *x = Character::Text(character));
    }

    /// Returns the state of the pixel at the given a pixel coordinate. Returns
    /// false if there is a text character here or if the coordinate is out of
    /// range.
    fn get_pixel(&self, coord: PixelCoord) -> bool {
        if let Some((index, x, y)) = self.translate_pix_to_char(coord) {
            if let Character::Bitmap(l) = self.get_character(index) {
                return l.peek(x, y);
            }
        }
        false
    }

    /// Draws a single pixel, given a pixel coordinate.
    fn set_pixel(&mut self, coord: PixelCoord, value: bool) {
        if let Some((index, x, y)) = self.translate_pix_to_char(coord) {
            if let Some(Character::Bitmap(l)) = self.get_character_mut(index) {
                l.poke(x, y, value);
            }
        }
    }

    /// Writes a string starting at the given coordinate.
    pub fn draw_string(&mut self, coord: InputCoord, text: &str) {
        if let Some((top_left, _, _)) = self.translate_in_to_char(coord) {
            let mut pos = top_left.clone();
            for char in text.chars() {
                if char == '\n' {
                    pos.x = top_left.x;
                    pos.y += 1;
                } else if !char.is_control() {
                    self.set_character(pos, char);
                    pos.x += 1;
                }
            }
        }
    }

    /// Draws a rectangle. Coordinate a must be less than coordinate b in both
    /// axes.
    pub fn draw_rect(&mut self, a: InputCoord, b: InputCoord) {
        let a = self.translate_in_to_pix(a);
        let b = self.translate_in_to_pix(b);
        for x in a.x..=b.x {
            self.set_pixel(PixelCoord { x, y: a.y }, true);
            self.set_pixel(PixelCoord { x, y: b.y }, true);
        }
        for y in a.y..=b.y {
            self.set_pixel(PixelCoord { x: a.x, y }, true);
            self.set_pixel(PixelCoord { x: b.x, y }, true);
        }
    }

    /// Draws a line.
    pub fn draw_line(&mut self, a: InputCoord, b: InputCoord) {
        let a = self.translate_in_to_pix(a);
        let b = self.translate_in_to_pix(b);
        for (x, y) in line_drawing::Bresenham::new((a.x, a.y), (b.x, b.y)) {
            self.set_pixel(PixelCoord { x, y }, true);
        }
    }

    /// Renders to a string with a given font.
    pub fn render(
        &self,
        output: &mut std::fmt::Formatter<'_>,
        font: &BitmapFont,
    ) -> std::fmt::Result {
        let mut it = self.data.iter();
        let mut done = false;
        let mut line = String::with_capacity(self.width);
        while !done {
            for _ in 0..self.width {
                match it.next() {
                    Some(Character::Text(c)) => {
                        line.push(*c);
                    }
                    Some(Character::Bitmap(l)) => {
                        line.push(font.translate(*l));
                    }
                    None => {
                        done = true;
                        break;
                    }
                }
            }
            writeln!(output, "{}", line.trim_end())?;
            line.clear();
        }
        if !self.footnotes.is_empty() {
            writeln!(output)?;
            for (index, footnote) in self.footnotes.iter().enumerate() {
                writeln!(output, "[{}]: {footnote}", index + 1)?;
            }
        }
        Ok(())
    }

    /// Renders to a string at x3/x2.5 times the scale, such that one subpixel
    /// equals half a character vertically, which can be perfectly represented
    /// using box-drawing characters. Allows visualization of the complete
    /// canvas without bitmap font heuristics.
    pub fn debug_render(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let in_width = self.width;
        let in_height = (self.data.len() + in_width - 1) / in_width;
        let out_width = self.width * 3;
        let out_height = in_height * 2 + (in_height + 1) / 2;
        let mut line = String::with_capacity(out_width);
        for y in 0..out_height {
            for x in 0..out_width {
                let upper = PixelCoord::new(x as i64, (y * 2) as i64);
                let lower = PixelCoord::new(x as i64, (y * 2 + 1) as i64);
                if x % 3 == 1 && (upper.y % 5 == 2 || lower.y % 5 == 2) {
                    if let Some((cc, _, _)) = self.translate_pix_to_char(upper) {
                        if let Character::Text(c) = self.get_character(cc) {
                            line.push(c);
                            continue;
                        }
                    }
                }
                let upper = self.get_pixel(upper);
                let lower = self.get_pixel(lower);
                line.push(match (upper, lower) {
                    (true, true) => '█',
                    (true, false) => '▀',
                    (false, true) => '▄',
                    (false, false) => ' ',
                });
            }
            writeln!(output, "{}", line.trim_end())?;
            line.clear();
        }
        Ok(())
    }
}

impl std::fmt::Display for Canvas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            self.debug_render(f)
        } else {
            self.render(f, &BitmapFont::default())
        }
    }
}

/// A character coordinate in an ASCII-art canvas.
pub type CharCoord = vector2d::Vector2D<usize>;

/// A pixel coordinate in an ASCII-art canvas.
pub type PixelCoord = vector2d::Vector2D<i64>;

/// A floating-point coordinate in an ASCII-art canvas.
pub type InputCoord = vector2d::Vector2D<f64>;
