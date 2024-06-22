use crate::paint::paint;
use anyhow::Result;
use dom::{css, html, layout::LayoutBox, style::to_styled_node};
use engine::{runtime::Runtime, DOM};
use js_sys::wasm_bindgen;
use paint::CanvasAPI;
use util::Point;
use wasm_bindgen::{prelude::*, JsValue};

mod paint;
mod util;

fn run() -> Result<()> {
    let html = r#"    
<body>
    <p>hello</p>
    <p class="inline">world</p>
    <p class="inline">:)</p>
    <p>this</p>
    <p class="inline">is</p>
    <p class="inline">inline</p>
    <div class="none">
        <p>this should not be shown</p>
    </div>
    <span id="result">hoge</span>
</body>
    "#;
    let mut dom = DOM.try_lock().unwrap();
    *dom = html::parse(&html);
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

    let style = css::parse(css)?;

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
pub fn render() -> Result<(), JsValue> {
    match run() {
        Ok(_) => log::info!("Success"),
        Err(e) => log::error!("{}", e),
    };
    Ok(())
}

#[wasm_bindgen]
pub fn exec_js() -> Result<(), JsValue> {
    let source = r#"
    document.getElementById("result").innerText = "fuga";
    document.getElementById("result").innerText;
    "#;
    let mut runtime = Runtime::new();
    let r = runtime.execute("", source);
    log::debug!("r: {:?}", r);
    Ok(())
}

// fn main() -> Result<()> {
//     setup_logger()?;
//     {
//         let mut html_file = File::open("../sample/sample.html")?;
//         let mut html = String::new();
//         html_file.read_to_string(&mut html)?;
//         let mut dom = DOM.try_lock().unwrap();
//         *dom = html::parse(&html);
//     }

//     // let mut renderer = Renderer::new(node);
//     // renderer.execute_inline_scripts();

//     let source = r#"
//     document.getElementById("result").innerText = "fuga";
//     document.getElementById("result").innerText;
//     "#;
//     let mut runtime = Runtime::new();
//     let r = runtime.execute("", source);
//     log::debug!("r: {:?}", r);

//     Ok(())
// }
