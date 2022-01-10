use crate::prelude::{string_to_cp437, Console, DrawBatch, FontCharType, Tile, ColoredTextSpans, to_cp437};
use bracket_color::prelude::{ColorPair, RGB, RGBA};
use bracket_geometry::prelude::{Point, PointF, Radians, Rect, RectF};
use std::cmp;
use std::collections::HashMap;
use std::convert::TryInto;
use std::prelude::rust_2021::FromIterator;

pub struct TextBlock {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    fg: RGBA,
    bg: RGBA,
    buffer: Vec<Tile>,
    cursor: (i32, i32),
    padding: RectF,
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
            padding: RectF::zero(),
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
            padding: RectF::zero(),
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

    pub fn render_to_draw_batch_fancy<ANGLE: Into<Radians>, Z: TryInto<i32>>(
        &self,
        draw_batch: &mut DrawBatch,
        offset: PointF,
        z_order: Z,
        rotation: ANGLE,
        scale: PointF,
    ) {
        let z_order = z_order.try_into().ok().expect("Must be i32 convertible");
        let rotation = rotation.into();
        for y in 0..self.height {
            let row_padding = (y + 1) as f32 * self.padding.y1 + y as f32 * self.padding.y2;
            for x in 0..self.width {
                let col_padding = (x + 1) as f32 * self.padding.x1 + x as f32 * self.padding.x2;
                draw_batch.set_fancy(
                    PointF::new(
                        col_padding + (x + self.x) as f32 + offset.x,
                        row_padding + (y + self.y) as f32 + offset.y,
                    ),
                    (&z_order).clone(),
                    (&rotation).clone(),
                    scale,
                    ColorPair::new(self.buffer[self.at(x, y)].fg, self.buffer[self.at(x, y)].bg),
                    self.buffer[self.at(x, y)].glyph,
                );
            }
        }
    }

    pub fn print(&mut self, text: &TextBuilder) -> Result<(), OutOfSpace> {
        for cmd in &text.commands {
            match cmd {
                CommandType::Text { block: t, wrap, center_x, center_y, colored } => {
                    let mut colors: HashMap<usize, RGBA> = HashMap::new();
                    let mut char_index: usize = 0;

                    let mut buf: Vec<char> = Vec::new();

                    if *colored {
                        let split_text = ColoredTextSpans::new(t);

                        for span in split_text.spans.iter() {
                            let fg = span.0;
                            for c in span.1.chars() {
                                if fg != self.fg {
                                    colors.insert(char_index, fg);
                                }
                                char_index += 1;
                                buf.push(c);
                            }
                        }
                    }

                    let parsed_text = String::from_iter(buf);

                    let text = if parsed_text.len() > 0 { &parsed_text } else { t };

                    let words = if *wrap {
                        text.split(' ').collect::<Vec<&str>>()
                    } else {
                        vec![text.as_str()]
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
                            let remaining_len = remaining.len() + self.horizontal_padding(remaining.len());


                            // First check if it has room to push the current chars
                            if let Some(line) = lines.last_mut() {
                                let line_len = self.len_with_padding(line);
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
                                let line_len = self.len_with_padding(line);
                                let next_len: usize = line_len + remaining_len;
                                if next_len <= width - 1 {
                                    &line.extend(remaining);
                                    word_index = word.len();
                                }
                            }
                        }
                    }

                    self.cursor.1 = if *center_y {
                        let total_height = self.height as usize + self.vertical_padding(self.height as usize);
                        let text_height = lines.len() + self.vertical_padding(lines.len());
                        (total_height as i32 / 2) - (text_height as i32 / 2)
                    } else {
                        0
                    };

                    let mut char_index = 0;

                    for (i, line) in lines.iter().enumerate() {
                        self.cursor.0 = if *center_x {
                            let total_width = self.width as usize + self.horizontal_padding(self.width as usize);
                            let text_width = self.len_with_padding(line);
                            (total_width as i32 / 2) - (text_width as i32 / 2)
                        } else {
                            0
                        };
                        for c in line {
                            let idx = self.at(self.cursor.0, self.cursor.1 + i as i32);
                            if idx < self.buffer.len() {
                                self.buffer[idx].glyph = *c;
                                self.buffer[idx].fg = *colors.get(&char_index).unwrap_or(&self.fg);
                                self.buffer[idx].bg = self.bg;
                                self.cursor.0 += 1;
                            } else {
                                return Err(OutOfSpace);
                            }
                            char_index += 1;
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

    pub fn set_padding(&mut self, rect: RectF) {
        self.padding = rect;
    }

    pub fn get_padding(&self) -> RectF {
        self.padding
    }

    pub fn vertical_padding(&self, num_lines: usize) -> usize {
        (num_lines as f32 * (self.padding.y1 + self.padding.y2)).ceil() as usize
    }

    pub fn horizontal_padding(&self, num_chars: usize) -> usize {
        (num_chars as f32 * (self.padding.x1 + self.padding.x2)).ceil() as usize
    }

    pub fn len_with_padding<T>(&self, vec: &Vec<T>) -> usize {
        vec.len() + self.horizontal_padding(vec.len())
    }
}

pub enum CommandType {
    Text { block: String, wrap: bool, center_x: bool, center_y: bool, colored: bool },
    NewLine {},
    Foreground { col: RGBA },
    Background { col: RGBA },
    Reset {},
}

pub struct TextBuilder {
    wrap: bool,
    center_x: bool,
    center_y: bool,
    colored: bool,
    commands: Vec<CommandType>,
}

impl TextBuilder {
    pub fn empty() -> TextBuilder {
        TextBuilder {
            wrap: false,
            center_x: false,
            center_y: false,
            colored: true,
            commands: Vec::new(),
        }
    }

    pub fn wrap(&mut self, wrap: bool) -> &mut Self {
        self.wrap = wrap;
        self
    }

    pub fn center_x(&mut self, center_x: bool) -> &mut Self {
        self.center_x = center_x;
        self
    }

    pub fn center_y(&mut self, center_y: bool) -> &mut Self {
        self.center_y = center_y;
        self
    }

    pub fn append(&mut self, text: &str) -> &mut Self {
        self.commands.push(CommandType::Text {
            block: text.to_string(),
            wrap: self.wrap, center_x: self.center_x, center_y: self.center_y, colored: self.colored
        });
        self
    }

    pub fn centered(&mut self, text: &str) -> &mut Self {
        let centered = self.center_x;
        self.center_x(true);
        self.append(text);
        self.center_x(centered);
        self
    }

    pub fn colored_mode(&mut self, colored: bool) -> &mut Self {
        self.colored = colored;
        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.wrap = false;
        self.center_x = false;
        self.center_y = false;
        self.colored = false;
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
