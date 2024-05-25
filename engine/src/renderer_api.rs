use dom::dom::Node;
use std::{cell::RefCell, path::PathBuf, rc::Rc};

pub struct RendererAPI {
    pub document_element: Rc<RefCell<Box<Node>>>,
}

impl RendererAPI {
    pub fn new(document_element: Rc<RefCell<Box<Node>>>) -> Self {
        Self { document_element }
    }

    pub fn rerender(&self) {
        let dom = self.document_element.borrow();
        dom.write_as_bin(&PathBuf::from("sample/sample.html.bin"))
            .unwrap();
    }
}
