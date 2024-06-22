use anyhow::Result;
use boa_engine::{class::Class, js_string, Context, Source};
use document::Document;
use dom::{dom::Node, html};
use element::Element;
use renderer::Renderer;
use renderer_api::RendererAPI;
use std::{cell::RefCell, fs::File, io::Read, path::PathBuf, rc::Rc, time::Duration};

mod document;
mod element;
mod renderer;
mod renderer_api;

#[derive(Debug)]
pub struct JavaScriptRuntime {
    context: boa_engine::Context,
    renderer_api: Rc<RendererAPI>,
    document_element: Rc<RefCell<Box<Node>>>,
}

impl JavaScriptRuntime {
    pub fn new(document_element: Rc<RefCell<Box<Node>>>, renderer_api: Rc<RendererAPI>) -> Self {
        let mut context = Context::default();
        context.register_global_class::<Document>().unwrap();
        context.register_global_class::<Element>().unwrap();

        // `create_document_object()` の返り値をグローバル変数 `document` に格納する
        let document = Document::from_data(Document, &mut context).unwrap();
        let global = context.global_object();
        global
            .set(js_string!("document"), document, false, &mut context)
            .unwrap();

        JavaScriptRuntime {
            renderer_api,
            context,
            document_element,
        }
    }

    /// `execute` runs a given source in the current context.
    pub fn execute(&mut self, _filename: &str, source: &str) -> Result<String, String> {
        match self.context.eval(Source::from_bytes(source)) {
            Ok(value) => Ok(value
                .to_string(&mut self.context)
                .unwrap()
                .to_std_string_escaped()),
            Err(error) => Err(error.to_string()),
        }
    }
}

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
    let mut html_file = File::open("sample/sample.html")?;
    let mut html = String::new();
    html_file.read_to_string(&mut html)?;
    let node = html::parse(&html);
    node.write_as_bin(&PathBuf::from("sample/sample.html.bin"))?;
    std::thread::sleep(Duration::from_secs(3));
    let mut renderer = Renderer::new(node);
    renderer.execute_inline_scripts();

    Ok(())
}
