use crate::prelude::{string_to_cp437, Console, DrawBatch, FontCharType, Tile};
use bracket_color::prelude::{ColorPair, RGB, RGBA};
use bracket_geometry::prelude::{Point, Rect};
use std::cmp;

pub struct TextBlock {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    fg: RGBA,
    bg: RGBA,
    buffer: Vec<Tile>,
    cursor: (i32, i32),
}

#[derive(Debug, Clone)]
pub struct OutOfSpace;

impl std::fmt::Display for OutOfSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Out of text-buffer space.")
    }
}

impl TextBlock {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> TextBlock {
        TextBlock {
            x,
            y,
            width,
            height,
            fg: RGBA::from_f32(1.0, 1.0, 1.0, 1.0),
            bg: RGBA::from_f32(0.0, 0.0, 0.0, 1.0),
            buffer: vec![
                Tile {
                    glyph: 0,
                    fg: RGBA::from_f32(1.0, 1.0, 1.0, 1.0),
                    bg: RGBA::from_f32(0.0, 0.0, 0.0, 1.0)
                };
                width as usize * height as usize
            ],
            cursor: (0, 0),
        }
    }

    pub fn new_with_color<COLOR, COLOR2>(
        x: i32, y: i32, width: i32, height: i32, fg: COLOR, bg: COLOR2
    ) -> TextBlock where COLOR: Into<RGBA>, COLOR2: Into<RGBA> {
        let color_pair = ColorPair { fg: fg.into(), bg: bg.into() };
        TextBlock {
            x,
            y,
            width,
            height,
            fg: color_pair.fg,
            bg: color_pair.bg,
            buffer: vec![
                Tile {
                    glyph: 0,
                    fg: color_pair.fg,
                    bg: color_pair.bg,
                };
                width as usize * height as usize
            ],
            cursor: (0, 0),
        }
    }

    pub fn fg<COLOR>(&mut self, fg: RGB)
    where
        COLOR: Into<RGBA>,
    {
        self.fg = fg.into();
    }

    pub fn bg<COLOR>(&mut self, bg: COLOR)
    where
        COLOR: Into<RGBA>,
    {
        self.bg = bg.into();
    }

    pub fn move_to(&mut self, x: i32, y: i32) {
        self.cursor = (x, y);
    }

    pub fn get_cursor(&self) -> Point {
        Point::from_tuple(self.cursor)
    }

    pub fn get_origin(&self) -> Point {
        Point::new(self.x, self.y)
    }

    pub fn set_origin(&mut self, origin: Point) {
        self.x = origin.x;
        self.y = origin.y;
    }

    fn at(&self, x: i32, y: i32) -> usize {
        ((y * self.width) + x) as usize
    }

    pub fn render(&self, mut console: impl AsMut<dyn Console>) {
        for y in 0..self.height {
            for x in 0..self.width {
                console.as_mut().set(
                    x + self.x,
                    y + self.y,
                    self.buffer[self.at(x, y)].fg,
                    self.buffer[self.at(x, y)].bg,
                    self.buffer[self.at(x, y)].glyph,
                );
            }
        }
    }

    pub fn render_to_draw_batch(&self, draw_batch: &mut DrawBatch) {
        for y in 0..self.height {
            for x in 0..self.width {
                draw_batch.set(
                    Point::new(x + self.x, y + self.y),
                    ColorPair::new(self.buffer[self.at(x, y)].fg, self.buffer[self.at(x, y)].bg),
                    self.buffer[self.at(x, y)].glyph,
                );
            }
        }
    }

    pub fn render_to_draw_batch_clip(&self, draw_batch: &mut DrawBatch, clip: &Rect) {
        for y in cmp::max(0, clip.y1)..cmp::min(self.height, clip.y2) {
            for x in cmp::max(0, clip.x1)..cmp::min(self.width, clip.x2) {
                draw_batch.set(
                    Point::new(x + self.x, y + self.y),
                    ColorPair::new(self.buffer[self.at(x, y)].fg, self.buffer[self.at(x, y)].bg),
                    self.buffer[self.at(x, y)].glyph,
                );
            }
        }
    }

    pub fn print(&mut self, text: &TextBuilder) -> Result<(), OutOfSpace> {
        for cmd in &text.commands {
            match cmd {
                CommandType::Text { block: t, wrap, center } => {
                    let words = if *wrap {
                        t.split(' ').collect::<Vec<&str>>()
                    } else {
                        vec![t.as_str()]
                    };

                    let all_words = words
                        .iter()
                        .map(|word| string_to_cp437(&word))
                        .collect::<Vec<Vec<FontCharType>>>();

                    let mut lines: Vec<Vec<FontCharType>> = vec![];
                    let width = self.width as usize;

                    for word in all_words {
                        if lines.len() == 0 {
                            lines.push(vec![]);
                        }

                        let mut word_index = 0;

                        if let Some(line) = lines.last_mut() {
                            if line.len() > 0 {
                                // TODO: "Clear character" instead of 32 (' ') here...
                                line.extend(&string_to_cp437(" "))
                            }
                        }

                        while word_index < word.len() {
                            let remaining = &word[word_index..];

                            // First check if it has room to push the current chars
                            if let Some(line) = lines.last_mut() {
                                let line_len = line.len();
                                let remaining_len = remaining.len();
                                let next_len: usize = line_len + remaining_len;

                                let overflow = next_len as i32 - (width as i32 - 1);
                                if overflow > 0 {
                                    if !*wrap {
                                        let end = word_index + (remaining_len as i32 - overflow) as usize;
                                        &line.extend(&word[word_index..end]);
                                        word_index = end;
                                    }
                                    lines.push(vec![]);
                                }
                            }

                            if let Some(line) = lines.last_mut() {
                                let line_len = line.len();
                                let remaining_len = remaining.len();
                                let next_len: usize = line_len + remaining_len;
                                if next_len <= width - 1 {
                                    &line.extend(remaining);
                                    word_index = word.len();
                                }
                            }
                        }
                    }

                    // Trim lines
                    for line in lines.iter_mut() {
                        if line.ends_with(&string_to_cp437(" ")) {
                            line.pop();
                        }
                    }

                    for (i, line) in lines.iter().enumerate() {
                        self.cursor.0 = if *center {
                            let text_width = line.len() as i32;
                            let half_width = text_width / 2;
                            (self.width / 2) - half_width
                        } else {
                            0
                        };
                        for c in line {
                            let idx = self.at(self.cursor.0, self.cursor.1 + i as i32);
                            if idx < self.buffer.len() {
                                self.buffer[idx].glyph = *c;
                                self.buffer[idx].fg = self.fg;
                                self.buffer[idx].bg = self.bg;
                                self.cursor.0 += 1;
                            } else {
                                return Err(OutOfSpace);
                            }
                        }
                    }
                }

                CommandType::NewLine {} => {
                    self.cursor.0 = 0;
                    self.cursor.1 += 1;
                }

                CommandType::Foreground { col } => self.fg = *col,
                CommandType::Background { col } => self.bg = *col,
                CommandType::Reset {} => {
                    self.cursor = (0, 0);
                    self.fg = RGBA::from_f32(1.0, 1.0, 1.0, 1.0);
                    self.bg = RGBA::from_f32(0.0, 0.0, 0.0, 1.0);
                }
            }
        }
        Ok(())
    }
}

pub enum CommandType {
    Text { block: String, wrap: bool, center: bool },
    NewLine {},
    Foreground { col: RGBA },
    Background { col: RGBA },
    Reset {},
}

pub struct TextBuilder {
    wrap: bool,
    center: bool,
    commands: Vec<CommandType>,
}

impl TextBuilder {
    pub fn empty() -> TextBuilder {
        TextBuilder {
            wrap: false,
            center: false,
            commands: Vec::new(),
        }
    }

    pub fn wrap(&mut self, wrap: bool) -> &mut Self {
        self.wrap = wrap;
        self
    }

    pub fn center(&mut self, center: bool) -> &mut Self {
        self.center = center;
        self
    }

    pub fn append(&mut self, text: &str) -> &mut Self {
        self.commands.push(CommandType::Text {
            block: text.to_string(), wrap: self.wrap, center: self.center
        });
        self
    }

    pub fn centered(&mut self, text: &str) -> &mut Self {
        let centered = self.center;
        self.center(true);
        self.append(text);
        self.center(centered);
        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.wrap = false;
        self.center = false;
        self.commands.push(CommandType::Reset {});
        self
    }
    pub fn ln(&mut self) -> &mut Self {
        self.commands.push(CommandType::NewLine {});
        self
    }
    pub fn fg<COLOR>(&mut self, col: COLOR) -> &mut Self
    where
        COLOR: Into<RGBA>,
    {
        self.commands
            .push(CommandType::Foreground { col: col.into() });
        self
    }
    pub fn bg<COLOR>(&mut self, col: COLOR) -> &mut Self
    where
        COLOR: Into<RGBA>,
    {
        self.commands
            .push(CommandType::Background { col: col.into() });
        self
    }
    pub fn line_wrap(&mut self, text: &str) -> &mut Self {
        let wrapped = self.wrap;
        self.wrap(true);
        self.append(text);
        self.wrap(wrapped);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{TextBlock, TextBuilder};

    #[test]
    fn textblock_ok() {
        let mut block = TextBlock::new(0, 0, 80, 25);

        let mut buf = TextBuilder::empty();
        buf.ln()
            .centered("Hello World")
            .line_wrap("The quick brown fox jumped over the lazy dog, and just kept on running in an attempt to exceed the console width.")
            .reset();

        assert!(block.print(&buf).is_ok());
    }

    #[test]
    fn textblock_wrap_error() {
        let mut block = TextBlock::new(0, 0, 80, 2);

        let mut buf = TextBuilder::empty();
        buf.ln()
            .centered("Hello World")
            .line_wrap("The quick brown fox jumped over the lazy dog, and just kept on running in an attempt to exceed the console width.")
            .line_wrap("The quick brown fox jumped over the lazy dog, and just kept on running in an attempt to exceed the console width.")
            .reset();

        assert!(block.print(&buf).is_err());
    }
}
