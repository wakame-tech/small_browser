use crate::{runtime::Runtime, DOM};
use dom::dom::{Node, NodeType};

pub struct Renderer {
    runtime: Runtime,
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
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new(),
        }
    }

    // Renderer が管理する DOM ツリー（self.document_element）内の JavaScript を実行する関数
    pub fn execute_inline_scripts(&mut self) {
        let scripts = {
            let document_element = DOM.lock().unwrap();
            collect_tag_inners(&document_element, "script".into()).join("\n")
        };
        log::debug!("scripts: {}", scripts);
        self.runtime.execute("(inline)", scripts.as_str()).unwrap();
    }
}
