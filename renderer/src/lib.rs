use crate::paint::paint;
use anyhow::Result;
use dom::{
    css,
    dom::{Node, NodeType},
    html,
    layout::LayoutBox,
    style::to_styled_node,
};
use engine::{runtime::Runtime, DOM};
use js_sys::wasm_bindgen;
use paint::CanvasAPI;
use util::Point;
use wasm_bindgen::{prelude::*, JsValue};

mod paint;
mod util;

fn collect_tag_inners(node: &Box<Node>, tag_name: &str) -> Vec<String> {
    if let NodeType::Element(ref element) = node.node_type {
        if element.tag_name.as_str() == tag_name {
            return vec![node.inner_text()];
        }
    }

    node.children
        .iter()
        .map(|child| collect_tag_inners(child, tag_name))
        .collect::<Vec<Vec<String>>>()
        .into_iter()
        .flatten()
        .collect()
}

fn execute_inline_scripts(runtime: &mut Runtime) -> Result<String, String> {
    let scripts = {
        let document_element = DOM.try_lock().unwrap();
        collect_tag_inners(&document_element, "script".into()).join("\n")
    };
    runtime.execute("(inline)", scripts.as_str())
}

fn run(html: &str, css: &str) -> Result<()> {
    {
        let mut dom = DOM.try_lock().unwrap();
        *dom = html::parse(&html);
    }

    let mut runtime = Runtime::new();
    let result = execute_inline_scripts(&mut runtime).map_err(|e| anyhow::anyhow!(e))?;
    log::info!("Result: {}", result);

    let style = css::parse(css)?;

    let dom = DOM.try_lock().unwrap();
    let Some(styled_node) = to_styled_node(&dom, &style) else {
        return Err(anyhow::anyhow!("Failed to style node"));
    };
    let layout_box = LayoutBox::new(styled_node);
    let canvas = CanvasAPI::new();
    canvas.clear();
    paint(&Point { x: 0., y: 0. }, &canvas, &layout_box);

    Ok(())
}

#[wasm_bindgen]
pub fn setup() {
    wasm_logger::init(wasm_logger::Config::default());
}

#[wasm_bindgen]
pub fn render(html: &str, css: &str) -> Result<(), JsValue> {
    match run(html, css) {
        Ok(_) => log::info!("Success"),
        Err(e) => log::error!("{}", e),
    };
    Ok(())
}
