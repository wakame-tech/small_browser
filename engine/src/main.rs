use anyhow::Result;
use dom::{
    dom::{Node, NodeType, Text},
    html,
};
use once_cell::sync::Lazy;
use runtime::Runtime;
use std::{fs::File, io::Read, sync::Mutex};

mod document;
mod element;
mod renderer;
mod runtime;

static DOM: Lazy<Mutex<Box<Node>>> = Lazy::new(|| {
    Mutex::new(Box::new(Node {
        node_type: NodeType::Text(Text {
            data: String::new(),
        }),
        children: vec![],
    }))
});

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| out.finish(format_args!("[{}] {}", record.level(), message)))
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

fn main() -> Result<()> {
    setup_logger()?;
    {
        let mut html_file = File::open("../sample/sample.html")?;
        let mut html = String::new();
        html_file.read_to_string(&mut html)?;
        let mut dom = DOM.try_lock().unwrap();
        *dom = html::parse(&html);
    }

    // let mut renderer = Renderer::new(node);
    // renderer.execute_inline_scripts();

    let source = r#"
    document.getElementById("result").innerText = "fuga";
    document.getElementById("result").innerText;
    "#;
    let mut runtime = Runtime::new();
    let r = runtime.execute("", source);
    log::debug!("r: {:?}", r);

    Ok(())
}
