use std::ops::Rem;
use crate::prelude::{to_cp437, Console, FlexiConsole};
use bracket_color::prelude::RGBA;

/// Draws a box, starting at x/y with the extents width/height using CP437 line characters
pub fn draw_box(
    console: &mut dyn Console,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
    fg: RGBA,
    bg: RGBA,
) {
    for y in sy..sy + height {
        for x in sx..sx + width {
            console.set(
                x,
                y,
                fg,
                bg,
                console.get_clear_glyph(),
            );
        }
    }

    console.set(sx, sy, fg, bg, to_cp437('┌'));
    console.set(sx + width, sy, fg, bg, to_cp437('┐'));
    console.set(sx, sy + height, fg, bg, to_cp437('└'));
    console.set(sx + width, sy + height, fg, bg, to_cp437('┘'));
    for x in sx + 1..sx + width {
        console.set(x, sy, fg, bg, to_cp437('─'));
        console.set(x, sy + height, fg, bg, to_cp437('─'));
    }
    for y in sy + 1..sy + height {
        console.set(sx, y, fg, bg, to_cp437('│'));
        console.set(sx + width, y, fg, bg, to_cp437('│'));
    }
}

pub fn draw_box_fancy(
    console: &mut FlexiConsole,
    sx: f32,
    sy: f32,
    width: i32,
    height: i32,
    fg: RGBA,
    bg: RGBA,
) {
    for y in 0..height {
        for x in 0..width {
            console.set_fancy_simple(
                x as f32 + sx,
                y as f32 + sy,
                fg,
                bg,
                console.get_clear_glyph(),
            );
        }
    }

    console.set_fancy_simple(sx, sy as f32, fg, bg, to_cp437('┌'));
    console.set_fancy_simple(sx + width as f32, sy as f32, fg, bg, to_cp437('┐'));
    console.set_fancy_simple(sx as f32, sy + height as f32, fg, bg, to_cp437('└'));
    console.set_fancy_simple(sx + width as f32, sy + height as f32, fg, bg, to_cp437('┘'));
    for x in 1..width {
        console.set_fancy_simple(sx + x as f32, sy as f32, fg, bg, to_cp437('─'));
        console.set_fancy_simple(sx + x as f32, sy + height as f32, fg, bg, to_cp437('─'));
    }
    for y in 1..height {
        console.set_fancy_simple(sx as f32, sy + y as f32, fg, bg, to_cp437('│'));
        console.set_fancy_simple(sx + width as f32, sy + y as f32, fg, bg, to_cp437('│'));
    }
}

/// Draw a single-lined box without filling in the middle
pub fn draw_hollow_box(
    console: &mut dyn Console,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
    fg: RGBA,
    bg: RGBA,
) {
    console.set(sx, sy, fg, bg, to_cp437('┌'));
    console.set(sx + width, sy, fg, bg, to_cp437('┐'));
    console.set(sx, sy + height, fg, bg, to_cp437('└'));
    console.set(sx + width, sy + height, fg, bg, to_cp437('┘'));
    for x in sx + 1..sx + width {
        console.set(x, sy, fg, bg, to_cp437('─'));
        console.set(x, sy + height, fg, bg, to_cp437('─'));
    }
    for y in sy + 1..sy + height {
        console.set(sx, y, fg, bg, to_cp437('│'));
        console.set(sx + width, y, fg, bg, to_cp437('│'));
    }
}

/// Draws a box, starting at x/y with the extents width/height using CP437 line characters
pub fn draw_box_double(
    console: &mut dyn Console,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
    fg: RGBA,
    bg: RGBA,
) {
    for y in sy..sy + height {
        for x in sx..sx + width {
            console.set(
                x,
                y,
                RGBA::from_f32(1.0, 1.0, 1.0, fg.a),
                RGBA::from_f32(0.0, 0.0, 0.0, fg.a),
                32,
            );
        }
    }

    console.set(sx, sy, fg, bg, to_cp437('╔'));
    console.set(sx + width, sy, fg, bg, to_cp437('╗'));
    console.set(sx, sy + height, fg, bg, to_cp437('╚'));
    console.set(sx + width, sy + height, fg, bg, to_cp437('╝'));
    for x in sx + 1..sx + width {
        console.set(x, sy, fg, bg, to_cp437('═'));
        console.set(x, sy + height, fg, bg, to_cp437('═'));
    }
    for y in sy + 1..sy + height {
        console.set(sx, y, fg, bg, to_cp437('║'));
        console.set(sx + width, y, fg, bg, to_cp437('║'));
    }
}

/// Draws a box, starting at x/y with the extents width/height using CP437 line characters
pub fn draw_hollow_box_double(
    console: &mut dyn Console,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
    fg: RGBA,
    bg: RGBA,
) {
    console.set(sx, sy, fg, bg, to_cp437('╔'));
    console.set(sx + width, sy, fg, bg, to_cp437('╗'));
    console.set(sx, sy + height, fg, bg, to_cp437('╚'));
    console.set(sx + width, sy + height, fg, bg, to_cp437('╝'));
    for x in sx + 1..sx + width {
        console.set(x, sy, fg, bg, to_cp437('═'));
        console.set(x, sy + height, fg, bg, to_cp437('═'));
    }
    for y in sy + 1..sy + height {
        console.set(sx, y, fg, bg, to_cp437('║'));
        console.set(sx + width, y, fg, bg, to_cp437('║'));
    }
}

/// Draws a horizontal progress bar
#[allow(clippy::too_many_arguments)]
pub fn draw_bar_horizontal(
    console: &mut dyn Console,
    sx: i32,
    sy: i32,
    width: i32,
    n: i32,
    max: i32,
    fg: RGBA,
    bg: RGBA,
) {
    let percent = n as f32 / max as f32;
    let fill_width = (percent * width as f32) as i32;
    for x in 0..width {
        if x <= fill_width {
            console.set(sx + x, sy, fg, bg, to_cp437('▓'));
        } else {
            console.set(sx + x, sy, fg, bg, to_cp437('░'));
        }
    }
}

/// Draws a vertical progress bar
#[allow(clippy::too_many_arguments)]
pub fn draw_bar_vertical(
    console: &mut dyn Console,
    sx: i32,
    sy: i32,
    height: i32,
    n: i32,
    max: i32,
    fg: RGBA,
    bg: RGBA,
) {
    let percent = n as f32 / max as f32;
    let fill_height = height - ((percent * height as f32) as i32);
    for y in 0..height {
        if y >= fill_height {
            console.set(sx, sy + y, fg, bg, to_cp437('▓'));
        } else {
            console.set(sx, sy + y, fg, bg, to_cp437('░'));
        }
    }
}
