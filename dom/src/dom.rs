use crate::html;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::OpenOptions, io::Write, path::PathBuf};
pub type AttrMap = HashMap<String, String>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<Box<Node>>,
}

impl Node {
    pub fn inner_text(&self) -> String {
        self.children
            .iter()
            .clone()
            .into_iter()
            .map(|node| match &node.node_type {
                NodeType::Text(t) => t.data.clone(),
                _ => node.inner_text(),
            })
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn set_inner_text(&mut self, text: &str) {
        self.children = vec![Box::new(Node {
            node_type: NodeType::Text(Text {
                data: text.to_string(),
            }),
            children: vec![],
        })];
    }

    pub fn inner_html(&self) -> String {
        self.children
            .iter()
            .map(|node| format!("{:?}", node))
            .collect::<Vec<_>>()
            .join("")
    }

    pub fn set_inner_html(&mut self, html: &str) {
        self.children = html::parse_raw(html.into());
    }

    pub fn get_element_by_id<'a>(self: &'a mut Box<Self>, id: &str) -> Option<&'a mut Box<Self>> {
        match self.node_type {
            NodeType::Element(ref e) => {
                if e.id().map(|eid| eid.to_string() == id).unwrap_or(false) {
                    return Some(self);
                }
            }
            _ => (),
        };
        self.children
            .iter_mut()
            .find_map(|child| child.get_element_by_id(id))
    }
}

use anyhow::Result;

impl Node {
    pub fn write_as_bin(&self, path: &PathBuf) -> Result<()> {
        // domのバイナリを作る
        let mut bin_file = OpenOptions::new().write(true).create(true).open(path)?;
        let bin = bincode::serialize(&self)?;
        log::debug!("bin: {:?} bytes", bin.len());
        bin_file.write_all(&bin)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    Element(Element),
    Text(Text),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Element {
    pub tag_name: String,
    pub attributes: AttrMap,
}

impl Element {
    pub fn new(name: String, attributes: AttrMap, children: Vec<Box<Node>>) -> Box<Node> {
        Box::new(Node {
            node_type: NodeType::Element(Element {
                tag_name: name,
                attributes: attributes,
            }),
            children,
        })
    }

    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    pub fn attributes(&self) -> Vec<(String, String)> {
        self.attributes
            .iter()
            .clone()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Text {
    pub data: String,
}

impl Text {
    pub fn new(text: String) -> Box<Node> {
        Box::new(Node {
            node_type: NodeType::Text(Text { data: text }),
            children: vec![],
        })
    }
}
