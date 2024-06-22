use dom::dom::{Node, NodeType, Text};
use once_cell::sync::Lazy;
use std::sync::Mutex;

mod document;
mod element;
pub mod runtime;

pub static DOM: Lazy<Mutex<Box<Node>>> = Lazy::new(|| {
    Mutex::new(Box::new(Node {
        node_type: NodeType::Text(Text {
            data: String::new(),
        }),
        children: vec![],
    }))
});
