use crate::paint::CanvasAPI;
use anyhow::Result;
use combine::error::ParseError;
use combine::parser::char::{newline, space};
use combine::{many, Parser, Stream};
use layout::LayoutBox;
use paint::paint;
use style::to_styled_node;
use util::Point;

pub mod css;
pub mod dom;
pub mod html;
pub mod layout;
pub mod paint;
pub mod style;
pub mod util;

fn blank<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    // 空白と改行を読み飛ばす
    many::<String, _, _>(space().or(newline())).map(|_| ())
}

fn run(html: &str, css: &str) -> Result<()> {
    let canvas = CanvasAPI::new();
    let node = html::parse(html);
    let style = css::parse(css)?;
    let Some(styled_node) = to_styled_node(&node, &style) else {
        return Err(anyhow::anyhow!("Failed to style node"));
    };
    let layout_box = LayoutBox::new(styled_node);
    paint(&Point { x: 0., y: 0. }, &canvas, &layout_box);
    Ok(())
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    let html = r#"<body>
    <p>hello</p>
    <p class="inline">world</p>
    <p class="inline">:)</p>
    <div class="none"><p>this should not be shown</p></div>
    <style>
        .none {
            display: none;
        }
        .inline {
            display: inline;
        }
    </style>
</body>"#;
    let css = r#"* {
        display: inline;
    }
    "#;
    match run(html, css) {
        Ok(_) => log::info!("Success"),
        Err(e) => log::error!("{}", e),
    };
}
