pub use winit::event::VirtualKeyCode;
mod init;
pub mod shader_strings;
pub use init::*;
mod events;
pub use events::*;
mod mainloop;
use crate::hal::ConsoleBacking;
pub use mainloop::*;
use parking_lot::Mutex;
use std::any::Any;
use crate::hal::scaler::{ScreenScaler};

pub type GlCallback = fn(&mut dyn Any, &glow::Context);

pub struct InitHints {
    pub vsync: bool,
    pub fullscreen: bool,
    pub frame_sleep_time: Option<f32>,
    pub desired_gutter: u32,
    pub resize_scaling: bool,
}

impl InitHints {
    pub fn new() -> Self {
        Self {
            vsync: true,
            fullscreen: false,
            frame_sleep_time: None,
            desired_gutter: 0,
            resize_scaling: false,
        }
    }
}

pub struct PlatformGL {
    pub gl: Option<glow::Context>,
    pub quad_vao: Option<glow::WebVertexArrayKey>,
    pub backing_buffer: Option<super::Framebuffer>,
    pub frame_sleep_time: Option<u64>,
    pub gl_callback: Option<GlCallback>,
    pub resize_scaling: bool,
    pub resize_request: Option<(u32, u32)>,
    pub screen_scaler: ScreenScaler,
    pub request_screenshot: Option<String>,
}

lazy_static! {
    pub static ref BACKEND: Mutex<PlatformGL> = Mutex::new(PlatformGL {
        gl: None,
        quad_vao: None,
        gl_callback: None,
        frame_sleep_time: None,
        resize_scaling: false,
        resize_request: None,
        backing_buffer: None,
        screen_scaler: ScreenScaler::default(),
        request_screenshot: None,
    });
}

unsafe impl Send for PlatformGL {}
unsafe impl Sync for PlatformGL {}

lazy_static! {
    pub(crate) static ref CONSOLE_BACKING: Mutex<Vec<ConsoleBacking>> = Mutex::new(Vec::new());
}

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}
