use artcore::{AppConfig, WebConfig};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run(width: u32, token_address: String, html_canvas_id: String) {
    let app_config = AppConfig {
        width: width as f32,
        initial_token_address: token_address,
    };
    let web_config = WebConfig { html_canvas_id };
    app_config.build(Some(web_config)).run();
}
