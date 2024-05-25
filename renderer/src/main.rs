use crate::paint::paint;
use anyhow::Result;
use dom::{css, dom::Node, layout::LayoutBox, style::to_styled_node};
use js_sys::wasm_bindgen;
use paint::CanvasAPI;
use util::Point;
use wasm_bindgen::{prelude::*, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};

mod paint;
mod util;

fn run(node: Box<Node>, css: &str) -> Result<()> {
    let style = css::parse(css)?;
    let Some(styled_node) = to_styled_node(&node, &style) else {
        return Err(anyhow::anyhow!("Failed to style node"));
    };
    let layout_box = LayoutBox::new(styled_node);
    let canvas = CanvasAPI::new();
    canvas.clear();
    paint(&Point { x: 0., y: 0. }, &canvas, &layout_box);
    Ok(())
}

async fn fetch_dom() -> Result<Node, JsValue> {
    let opts = RequestInit::new();
    let url = format!("http://127.0.0.1:8000");
    let request = Request::new_with_str_and_init(&url, &opts)?;
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;

    // `resp_value` is a `Response` object.
    assert!(resp_value.is_instance_of::<Response>());
    let resp: Response = resp_value.dyn_into().unwrap();

    // Convert this other `Promise` into a rust `Future`.
    let buf = JsFuture::from(resp.array_buffer()?).await?;
    let array = js_sys::Uint8Array::new(&buf);
    let bytes = array.to_vec();
    let node: Node = bincode::deserialize(bytes.as_slice())
        .map_err(|_| JsValue::from("Failed to deserialize"))?;
    Ok(node)
}

#[wasm_bindgen]
pub async fn hoge() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    let node = Box::new(fetch_dom().await?);
    let css = r#"
script {
    display: none;
}
p, div {
    display: block;
}
    .none {
            display: none;
        }
        .inline {
            display: inline;
        }"#;
    // let css = r#"* {
    //     display: inline;
    // }"#;
    match run(node, css) {
        Ok(_) => log::info!("Success"),
        Err(e) => log::error!("{}", e),
    };
    Ok(())
}

fn main() {}
