mod framebuffer;

pub use framebuffer::*;
mod shader;
pub use shader::*;
mod font;
pub use font::*;
mod quadrender;
pub use quadrender::*;
mod vertex_array_helper;
pub(crate) use vertex_array_helper::*;
mod backing;
pub(crate) use backing::*;
mod glerror;
pub(crate) use glerror::*;

#[cfg(not(target_arch = "wasm32"))]
mod types_native;

#[cfg(not(target_arch = "wasm32"))]
pub use types_native::*;

#[cfg(target_arch = "wasm32")]
mod types_wasm;

#[cfg(target_arch = "wasm32")]
pub use types_wasm::*;

use instant::Instant;
use glow::HasContext;
use parking_lot::lock_api::MutexGuard;
use parking_lot::RawMutex;
use crate::hal::*;
use crate::prelude::{BTerm, GameState, BACKEND_INTERNAL};

/// Internal handling of the main loop.
pub fn tock<GS: GameState>(
    bterm: &mut BTerm,
    scale_factor: f32,
    gamestate: &mut GS,
    frames: &mut i32,
    prev_seconds: &mut u64,
    prev_ms: &mut u128,
    now: &Instant,
) {
    // Check that the console backings match our actual consoles
    check_console_backing();

    let now_seconds = now.elapsed().as_secs();
    *frames += 1;

    if now_seconds > *prev_seconds {
        bterm.fps = *frames as f32 / (now_seconds - *prev_seconds) as f32;
        *frames = 0;
        *prev_seconds = now_seconds;
    }

    let now_ms = now.elapsed().as_millis();
    if now_ms > *prev_ms {
        bterm.frame_time_ms = (now_ms - *prev_ms) as f32;
        *prev_ms = now_ms;
    }

    // Console structure - doesn't really have to be every frame...
    rebuild_consoles();

    // Bind to the backing buffer
    {
        let be = BACKEND.lock();
        be.backing_buffer
            .as_ref()
            .unwrap()
            .bind(be.gl.as_ref().unwrap());
        unsafe {
            be.gl.as_ref().unwrap().viewport(0, 0, be.screen_scaler.logical_size.0 as i32, be.screen_scaler.logical_size.1 as i32);
        }
    }

    // Clear the backing buffer
    unsafe {
        clear(&BACKEND.lock());
    }

    // Run the main loop
    gamestate.tick(bterm);

    // Tell each console to draw itself
    render_consoles().unwrap();

    // If there is a GL callback, call it now
    {
        let be = BACKEND.lock();
        if let Some(callback) = be.gl_callback.as_ref() {
            let gl = be.gl.as_ref().unwrap();
            callback(gamestate, gl);
        }
    }

    {
        // Now we return to the primary screen
        let be = BACKEND.lock();
        be.backing_buffer
            .as_ref()
            .unwrap()
            .default(be.gl.as_ref().unwrap());
        unsafe {
            // And clear it, resetting the viewport
            be.gl.as_ref().unwrap().viewport(0, 0, be.screen_scaler.physical_size.0 as i32, be.screen_scaler.physical_size.1 as i32);
            clear(&be);

            let bi = BACKEND_INTERNAL.lock();
            if bterm.post_scanlines {
                bi.shaders[3].useProgram(be.gl.as_ref().unwrap());
                bi.shaders[3].setVec3(
                    be.gl.as_ref().unwrap(),
                    "screenSize",
                    scale_factor * bterm.width_pixels as f32,
                    scale_factor * bterm.height_pixels as f32,
                    0.0,
                );
                bi.shaders[3].setBool(be.gl.as_ref().unwrap(), "screenBurn", bterm.post_screenburn);
                bi.shaders[3].setVec3(
                    be.gl.as_ref().unwrap(),
                    "screenBurnColor",
                    bterm.screen_burn_color.r,
                    bterm.screen_burn_color.g,
                    bterm.screen_burn_color.b,
                );
            } else {
                bi.shaders[2].useProgram(be.gl.as_ref().unwrap());
            }
            be.gl
                .as_ref()
                .unwrap()
                .bind_vertex_array(Some(be.quad_vao.unwrap()));
            be.gl.as_ref().unwrap().bind_texture(
                glow::TEXTURE_2D,
                Some(be.backing_buffer.as_ref().unwrap().texture),
            );
            be.gl.as_ref().unwrap().draw_arrays(glow::TRIANGLES, 0, 6);
        }
    }
}

unsafe fn clear(be: &MutexGuard<RawMutex, PlatformGL>) {
    be.gl.as_ref().unwrap().clear_color(0.0, 0.0, 0.0, 1.0);
    be.gl.as_ref().unwrap().clear(glow::COLOR_BUFFER_BIT);
}
