use anyhow::Result;
use combine::error::ParseError;
use combine::parser::char::{newline, space};
use combine::{many, Parser, Stream};
use layout::LayoutBox;
use std::fs::OpenOptions;
use std::io::Write;
use style::to_styled_node;

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
    let node = html::parse(html);
    let style = css::parse(css)?;
    let Some(styled_node) = to_styled_node(&node, &style) else {
        return Err(anyhow::anyhow!("Failed to style node"));
    };
    let layout_box = LayoutBox::new(styled_node);

    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::paint::CanvasAPI;
        use paint::paint;
        use util::Point;

        let mut f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open("./tree.txt")?;
        f.write_all(layout_box.debug(0).as_bytes())?;
    }

    #[cfg(target_arch = "wasm32")]
    {
        let canvas = CanvasAPI::new();
        paint(&Point { x: 0., y: 0. }, &canvas, &layout_box);
    }
    Ok(())
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    wasm_logger::init(wasm_logger::Config::default());

    let html = r#"<body>
    <p>hello</p>
    <p class="inline">world</p>
    <p class="inline">:)</p>
    <p>this</p>
    <p class="inline">is</p>
    <p class="inline">inline</p>
    <div class="none"><p>this should not be shown</p></div>
</body>"#;

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
    match run(html, css) {
        Ok(_) => log::info!("Success"),
        Err(e) => log::error!("{}", e),
    };
}
