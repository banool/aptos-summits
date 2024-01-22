use core::{AppConfig, WebConfig};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run(width: u32, token_address: String, html_canvas_id: String) {
    let app_config = AppConfig {
        width: width as f32,
        token_address,
    };
    let web_config = WebConfig { html_canvas_id };
    app_config.run(Some(web_config));
}
