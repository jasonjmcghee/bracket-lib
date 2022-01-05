/// This module handles binding external web elements, for example movement buttons.
/// Based heavily on Zireael07's pull request, but modified to be significantly more generic.
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub static mut GLOBAL_BUTTON: Option<String> = None;
pub static mut GLOBAL_SIZE: Option<(u32, u32)> = None;

#[allow(dead_code)]
pub fn register_html_button<S: ToString>(element_id: S) {
    let button = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id(&element_id.to_string())
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();

    let html_callback = Closure::wrap(Box::new(|e: web_sys::Event| {
        on_external_element_click(e.clone());
    }) as Box<dyn FnMut(_)>);

    button.set_onclick(Some(html_callback.as_ref().unchecked_ref()));
    html_callback.forget();
}

#[allow(dead_code)]
pub fn register_on_resize() {
    // Call it once
    on_external_resize();

    let html_callback = Closure::wrap(Box::new(move |_e: web_sys::Event| {
        on_external_resize();
    }) as Box<dyn FnMut(_)>);

    web_sys::window().unwrap().set_onresize(Some(html_callback.as_ref().unchecked_ref()));
    html_callback.forget();
}

#[allow(dead_code)]
pub fn on_external_element_click(event: web_sys::Event) {
    //set_command(Command::MoveLeft);
    unsafe {
        GLOBAL_BUTTON = Some(
            event
                .target()
                .unwrap()
                .dyn_into::<web_sys::HtmlElement>()
                .unwrap()
                .id(),
        );
        //crate::console::log(format!("{}", GLOBAL_BUTTON.clone().unwrap()));
    }
}

#[allow(dead_code)]
pub fn on_external_resize() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas_dom = document.get_element_by_id("canvas").unwrap();
    let canvas = canvas_dom.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    let window_width = window.inner_width().unwrap().as_f64().unwrap();
    let window_height = window.inner_height().unwrap().as_f64().unwrap();

    let ratio = canvas.height() as f64 / canvas.width() as f64;
    let raw_height = f64::min(window_width * ratio, window_height);
    let raw_width = raw_height / ratio;

    let pixel_ratio = window.device_pixel_ratio();

    let height = f64::round(raw_height * pixel_ratio);
    let width = f64::round(raw_width * pixel_ratio);

    canvas.set_width(width as u32);
    canvas.set_height(height as u32);

    let style = canvas.style();

    style.set_property("width", &format!("{}px", raw_width as u32)).unwrap();
    style.set_property("height", &format!("{}px", raw_height as u32)).unwrap();

    unsafe {
        GLOBAL_SIZE = Some((width as u32, height as u32));
    }
}