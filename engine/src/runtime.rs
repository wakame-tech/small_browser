use crate::{document::Document, element::Element};
use anyhow::Result;
use boa_engine::{class::Class, js_string, Context, Source};

#[derive(Debug)]
pub struct Runtime {
    context: boa_engine::Context,
}

impl Runtime {
    pub fn new() -> Self {
        let mut context = Context::default();
        context.register_global_class::<Document>().unwrap();
        context.register_global_class::<Element>().unwrap();

        // `create_document_object()` の返り値をグローバル変数 `document` に格納する
        let document = Document::from_data(Document, &mut context).unwrap();
        let global = context.global_object();
        global
            .set(js_string!("document"), document, false, &mut context)
            .unwrap();

        Runtime { context }
    }

    /// `execute` runs a given source in the current context.
    pub fn execute(&mut self, _filename: &str, source: &str) -> Result<String, String> {
        match self.context.eval(Source::from_bytes(source)) {
            Ok(value) => {
                let value = value
                    .to_string(&mut self.context)
                    .unwrap()
                    .to_std_string_escaped();
                Ok(value)
            }
            Err(error) => Err(error.to_string()),
        }
    }
}
