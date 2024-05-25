use crate::{renderer_api::RendererAPI, JavaScriptRuntime};
use dom::dom::{Node, NodeType};
use std::{cell::RefCell, rc::Rc};

pub struct Renderer {
    document_element: Rc<RefCell<Box<Node>>>,
    js_runtime_instance: JavaScriptRuntime,
}

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

impl Renderer {
    pub fn new(node: Box<Node>) -> Self {
        let document_element = Rc::new(RefCell::new(node));
        Self {
            document_element: document_element.clone(),
            js_runtime_instance: JavaScriptRuntime::new(
                document_element.clone(),
                Rc::new(RendererAPI {
                    document_element: document_element.clone(),
                }),
            ),
        }
    }

    // Renderer が管理する DOM ツリー（self.document_element）内の JavaScript を実行する関数
    pub fn execute_inline_scripts(&mut self) {
        let scripts = {
            let document_element = self.document_element.borrow();
            collect_tag_inners(&document_element, "script".into()).join("\n")
        };
        log::debug!("scripts: {}", scripts);
        self.js_runtime_instance
            .execute("(inline)", scripts.as_str())
            .unwrap();
    }
}
