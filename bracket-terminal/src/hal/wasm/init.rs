use crate::prelude::{BTerm, InitHints, BACKEND_INTERNAL};
use crate::BResult;

pub fn init_raw<S: ToString>(
    width_pixels: u32,
    height_pixels: u32,
    _window_title: S,
    platform_hints: InitHints,
) -> BResult<BTerm> {
    use super::super::*;
    use super::*;
    use wasm_bindgen::JsCast;
    use web_sys::console;

    let document = web_sys::window().unwrap().document().unwrap();
    let div = document.query_selector("#canvas-wrapper").unwrap().unwrap();
    let canvas = document.create_element("canvas").unwrap();
    canvas.set_attribute("id", "canvas").unwrap();

    console::log_2(&"Logging arbitrary values looks like".into(), &canvas.clone().into());

    div.append_child(&canvas).unwrap();

    let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    canvas.set_width(width_pixels);
    canvas.set_height(height_pixels);

    super::bind_wasm_events(&canvas);

    let webgl2_context = canvas
        .get_context("webgl2")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::WebGl2RenderingContext>()
        .unwrap();
    webgl2_context
        .get_extension("EXT_color_buffer_float")
        .expect("Unable to add extensions");

    let gl = glow::Context::from_webgl2_context(webgl2_context);

    // Load our basic shaders
    let mut shaders: Vec<Shader> = Vec::new();

    shaders.push(Shader::new(
        &gl,
        shader_strings::CONSOLE_WITH_BG_VS,
        shader_strings::CONSOLE_WITH_BG_FS,
    ));
    shaders.push(Shader::new(
        &gl,
        shader_strings::CONSOLE_NO_BG_VS,
        shader_strings::CONSOLE_NO_BG_FS,
    ));
    shaders.push(Shader::new(
        &gl,
        shader_strings::BACKING_VS,
        shader_strings::BACKING_FS,
    ));
    shaders.push(Shader::new(
        &gl,
        shader_strings::SCANLINES_VS,
        shader_strings::SCANLINES_FS,
    ));
    shaders.push(Shader::new(
        &gl,
        shader_strings::FANCY_CONSOLE_VS,
        shader_strings::FANCY_CONSOLE_FS,
    ));
    shaders.push(Shader::new(
        &gl,
        shader_strings::SPRITE_CONSOLE_VS,
        shader_strings::SPRITE_CONSOLE_FS,
    ));

    let quad_vao = setup_quad(&gl);

    let mut scaler = ScreenScaler::new(platform_hints.desired_gutter, width_pixels, height_pixels);
    let initial_dpi_factor = web_sys::window().unwrap().device_pixel_ratio();
    scaler.change_logical_size(width_pixels, height_pixels, initial_dpi_factor as f32);
    let backing_fbo = Framebuffer::build_fbo(
        &gl,
        scaler.logical_size.0 as i32,
        scaler.logical_size.1 as i32,
    );

    let mut be = BACKEND.lock();
    be.gl = Some(gl);
    be.quad_vao = Some(quad_vao);
    be.frame_sleep_time = crate::hal::convert_fps_to_wait(platform_hints.frame_sleep_time);
    be.backing_buffer = Some(backing_fbo);
    be.screen_scaler = scaler;
    be.resize_scaling = platform_hints.resize_scaling;

    BACKEND_INTERNAL.lock().shaders = shaders;

    Ok(BTerm {
        width_pixels,
        height_pixels,
        original_width_pixels: width_pixels,
        original_height_pixels: height_pixels,
        fps: 0.0,
        frame_time_ms: 0.0,
        active_console: 0,
        key: None,
        mouse_pos: (0, 0),
        left_click: false,
        shift: false,
        alt: false,
        control: false,
        web_button: None,
        quitting: false,
        post_scanlines: false,
        post_screenburn: false,
        screen_burn_color: bracket_color::prelude::RGB::from_f32(0.0, 1.0, 1.0),
    })
}
