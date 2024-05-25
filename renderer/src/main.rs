fn run(html: &str, css: &str) -> Result<()> {
    let node = html::parse(html);
    let style = css::parse(css)?;
    let Some(styled_node) = to_styled_node(&node, &style) else {
        return Err(anyhow::anyhow!("Failed to style node"));
    };
    let layout_box = LayoutBox::new(styled_node);

    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open("./tree.txt")?;
        f.write_all(layout_box.debug(0).as_bytes())?;
    }

    #[cfg(target_arch = "wasm32")]
    {
        use crate::paint::CanvasAPI;
        use paint::paint;
        use util::Point;
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
