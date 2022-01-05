use super::events::*;
use super::*;
use crate::hal::*;
use crate::prelude::{BTerm, GameState, BACKEND_INTERNAL, INPUT, BEvent};
use crate::{clear_input_state, BResult, gl_error_wrap};
use glow::HasContext;
use std::cell::RefCell;
use std::rc::Rc;
use bracket_geometry::prelude::Point;
use instant::Instant;
use wasm_bindgen::{JsCast};

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn run(mut draw_frame: impl FnMut() + 'static) {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        draw_frame();

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn largest_active_font() -> (u32, u32) {
    let bi = BACKEND_INTERNAL.lock();
    let mut max_width = 0;
    let mut max_height = 0;
    bi.consoles.iter().for_each(|c| {
        let size = bi.fonts[c.font_index].tile_size;
        if size.0 > max_width {
            max_width = size.0;
        }
        if size.1 > max_height {
            max_height = size.1;
        }
    });
    (max_width, max_height)
}

struct WindowSize {
    width: u32,
    height: u32,
}

fn on_resize(
    bterm: &mut BTerm,
    physical_size: WindowSize,
    dpi_scale_factor: f64,
    send_event: bool,
) {
    let font_max_size = largest_active_font();
    //println!("{:#?}", physical_size);
    INPUT.lock().set_scale_factor(dpi_scale_factor);
    let mut be = BACKEND.lock();
    be.screen_scaler.change_physical_size_smooth(physical_size.width, physical_size.height, dpi_scale_factor as f32, font_max_size);
    let (l, r, t, b) = be.screen_scaler.get_backing_buffer_output_coordinates();
    be.quad_vao = Some(setup_quad_gutter(be.gl.as_ref().unwrap(), l, r, t, b));
    if send_event {
        bterm.resize_pixels(
            physical_size.width as u32,
            physical_size.height as u32,
            be.resize_scaling,
        );
    }
    let gl = be.gl.as_ref().unwrap();
    unsafe {
        gl_error_wrap!(
            gl,
            gl.viewport(
                0,
                0,
                physical_size.width as i32,
                physical_size.height as i32,
            )
        );
    }
    /*let new_fb = Framebuffer::build_fbo(
        gl,
        physical_size.width as i32,
        physical_size.height as i32
    )?;
    be.backing_buffer = Some(new_fb);*/
    bterm.on_event(BEvent::Resized {
        new_size: Point::new(be.screen_scaler.available_width, be.screen_scaler.available_height),
        dpi_scale_factor: dpi_scale_factor as f32,
    });

    let mut bit = BACKEND_INTERNAL.lock();
    if be.resize_scaling && send_event {
        // Framebuffer must be rebuilt because render target changed
        let new_fb = Framebuffer::build_fbo(
            gl,
            be.screen_scaler.available_width as i32,
            be.screen_scaler.available_height as i32
        );
        be.backing_buffer = Some(new_fb);
        be.screen_scaler.logical_size.0 = be.screen_scaler.available_width;
        be.screen_scaler.logical_size.1 = be.screen_scaler.available_height;

        let num_consoles = bit.consoles.len();
        for i in 0..num_consoles {
            let font_size = bit.fonts[bit.consoles[i].font_index].tile_size;
            let chr_w = be.screen_scaler.available_width / font_size.0;
            let chr_h = be.screen_scaler.available_height / font_size.1;
            bit.consoles[i].console.set_char_size(chr_w, chr_h);
        }
    }
}

pub fn main_loop<GS: GameState>(mut bterm: BTerm, mut gamestate: GS) -> BResult<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    {
        let be = BACKEND.lock();
        let gl = be.gl.as_ref().unwrap();
        let mut bit = BACKEND_INTERNAL.lock();
        for f in bit.fonts.iter_mut() {
            f.setup_gl_texture(gl)?;
        }

        for s in bit.sprite_sheets.iter_mut() {
            let mut f = Font::new(&s.filename.to_string(), 1, 1, (1, 1));
            f.setup_gl_texture(gl)?;
            s.backing = Some(Rc::new(Box::new(f)));
        }
    }

    let now = instant::Instant::now();
    let mut prev_seconds = now.elapsed().as_secs();
    let mut prev_ms = now.elapsed().as_millis();
    let mut frames = 0;

    // Prepare for resize events
    register_on_resize();

    run(move || {
        // Read in event results
        let scale_factor= {
            BACKEND.lock().screen_scaler.scale_factor
        };
        unsafe {
            bterm.key = GLOBAL_KEY;
            bterm.left_click = GLOBAL_LEFT_CLICK;
            bterm.shift = GLOBAL_MODIFIERS.0;
            bterm.control = GLOBAL_MODIFIERS.1;
            bterm.alt = GLOBAL_MODIFIERS.2;
            bterm.web_button = GLOBAL_BUTTON.clone();
            if let Some((width, height)) = GLOBAL_SIZE {
                on_resize(&mut bterm, WindowSize { width, height }, scale_factor as f64, true);
                GLOBAL_SIZE = None;
            }
            bterm.mouse_pos = (
                (GLOBAL_MOUSE_POS.0 as f32 * scale_factor) as i32,
                (GLOBAL_MOUSE_POS.1 as f32 * scale_factor) as i32,
            );
            bterm.on_mouse_position(bterm.mouse_pos.0 as f64, bterm.mouse_pos.1 as f64);
        }

        // Call the tock function
        tock(
            &mut bterm,
            scale_factor,
            &mut gamestate,
            &mut frames,
            &mut prev_seconds,
            &mut prev_ms,
            &now,
        );

        // Clear any input
        clear_input_state(&mut bterm);
        unsafe {
            GLOBAL_KEY = None;
            GLOBAL_MODIFIERS = (false, false, false);
            GLOBAL_LEFT_CLICK = false;
            GLOBAL_BUTTON = None;
            GLOBAL_SIZE = None;
        }
    });

    Ok(())
}

/// Internal handling of the main loop.
fn tock<GS: GameState>(
    bterm: &mut BTerm,
    scale_factor: f32,
    gamestate: &mut GS,
    frames: &mut i32,
    prev_seconds: &mut u64,
    prev_ms: &mut u128,
    now: &Instant,
) {
    gl_common::tock(bterm, scale_factor, gamestate, frames, prev_seconds, prev_ms, now);

    // Screenshot handler
    {
        let mut be = BACKEND.lock();
        if be.request_screenshot.is_some() {
            let document = web_sys::window().unwrap().document().unwrap();
            let canvas_dom = document.query_selector("canvas").unwrap().unwrap();
            let canvas = canvas_dom.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
            let screenshot = canvas.to_data_url().unwrap();
            let link = document.create_element("a").unwrap()
                .dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
            link.set_href(&screenshot);
            link.set_download("screenshot.png");
            link.click();
        }
        be.request_screenshot = None;
    }
}