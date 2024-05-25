use super::{dom::NodeType, style::StyledNode};
use crate::style::{Display, PropertyMap};
use core::fmt;

#[derive(Debug, PartialEq)]
pub struct LayoutBox<'a> {
    pub box_type: BoxType<'a>,
    pub children: Vec<LayoutBox<'a>>,
}

impl<'a> LayoutBox<'a> {
    fn anonymous_block() -> Self {
        Self {
            box_type: BoxType::AnonymousBox,
            children: vec![],
        }
    }

    pub fn new(snode: StyledNode<'a>) -> Self {
        let mut root = Self {
            box_type: match snode.display() {
                Display::Inline => BoxType::InlineBox(BoxProps::from(&snode)),
                Display::Block => BoxType::BlockBox(BoxProps::from(&snode)),
                Display::None => unreachable!(),
            },
            children: vec![],
        };

        for child in snode.children {
            let child = LayoutBox::new(child);
            match child.box_type {
                BoxType::BlockBox(_) => root.children.push(child),
                BoxType::InlineBox(_) => root.get_inline_container().children.push(child),
                BoxType::AnonymousBox => {}
            }
        }

        root
    }

    fn get_inline_container(&mut self) -> &mut LayoutBox<'a> {
        match self.box_type {
            BoxType::InlineBox(_) | BoxType::AnonymousBox => self,
            BoxType::BlockBox(_) => {
                if self
                    .children
                    .last()
                    .map(|last| last.box_type != BoxType::AnonymousBox)
                    .unwrap_or(true)
                {
                    self.children.push(LayoutBox::anonymous_block())
                };
                self.children.last_mut().unwrap()
            }
        }
    }

    pub fn debug(&self, nest: usize) -> String {
        let pad = " ".repeat(nest * 2);
        let mut s = match &self.box_type {
            BoxType::BlockBox(p) => format!("{}- BlockBox {}", pad, p),
            BoxType::InlineBox(p) => format!("{}- InlineBox {}", pad, p),
            BoxType::AnonymousBox => format!("{}- AnonymousBox", pad),
        };
        s += "\n";
        for child in &self.children {
            s += child.debug(nest + 1).as_str();
        }
        s
    }
}

#[derive(Debug, PartialEq)]
pub enum BoxType<'a> {
    BlockBox(BoxProps<'a>),
    InlineBox(BoxProps<'a>),
    AnonymousBox,
}

impl BoxType<'_> {
    pub fn get_props(&self) -> Option<&BoxProps> {
        match self {
            BoxType::BlockBox(p) | BoxType::InlineBox(p) => Some(p),
            BoxType::AnonymousBox => None,
        }
    }

    pub fn is_inline(&self) -> bool {
        match self {
            BoxType::InlineBox(_) | BoxType::AnonymousBox => true,
            _ => false,
        }
    }
}

impl fmt::Display for BoxType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BoxType::BlockBox(p) => write!(f, "BlockBox {}", p),
            BoxType::InlineBox(p) => write!(f, "InlineBox {}", p),
            BoxType::AnonymousBox => write!(f, "AnonymousBox"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct BoxProps<'a> {
    pub node_type: &'a NodeType,
    pub properties: PropertyMap,
}

impl fmt::Display for BoxProps<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.node_type {
            NodeType::Element(e) => write!(f, "tag: {}", e.tag_name),
            NodeType::Text(t) => write!(f, "text: {}", t.data),
        }
    }
}

impl<'a> BoxProps<'a> {
    fn from(snode: &StyledNode<'a>) -> Self {
        Self {
            node_type: snode.node_type,
            properties: snode.properties.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{css::CSSValue, dom::Element};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_to_layout_box() {
        let block = [(
            "display".to_string(),
            CSSValue::Keyword("block".to_string()),
        )];
        let inline = [(
            "display".to_string(),
            CSSValue::Keyword("inline".to_string()),
        )];

        let node = NodeType::Element(Element {
            tag_name: "div".into(),
            attributes: [].iter().cloned().collect(),
        });
        let snode = StyledNode {
            node_type: &node,
            properties: block.iter().cloned().collect(),
            children: vec![
                StyledNode {
                    node_type: &node,
                    properties: block.iter().cloned().collect(),
                    children: vec![],
                },
                StyledNode {
                    node_type: &node,
                    properties: inline.iter().cloned().collect(),
                    children: vec![
                        StyledNode {
                            node_type: &node,
                            properties: block.iter().cloned().collect(),
                            children: vec![],
                        },
                        StyledNode {
                            node_type: &node,
                            properties: block.iter().cloned().collect(),
                            children: vec![],
                        },
                    ],
                },
                StyledNode {
                    node_type: &node,
                    properties: inline.iter().cloned().collect(),
                    children: vec![],
                },
                StyledNode {
                    node_type: &node,
                    properties: block.iter().cloned().collect(),
                    children: vec![],
                },
            ],
        };

        assert_eq!(
            LayoutBox::new(snode),
            LayoutBox {
                box_type: BoxType::BlockBox(BoxProps {
                    node_type: &node,
                    properties: block.iter().cloned().collect(),
                }),
                children: vec![
                    LayoutBox {
                        box_type: BoxType::BlockBox(BoxProps {
                            node_type: &node,
                            properties: block.iter().cloned().collect(),
                        }),
                        children: vec![],
                    },
                    LayoutBox {
                        box_type: BoxType::AnonymousBox,
                        children: vec![
                            LayoutBox {
                                box_type: BoxType::InlineBox(BoxProps {
                                    node_type: &node,
                                    properties: inline.iter().cloned().collect(),
                                }),
                                children: vec![
                                    LayoutBox {
                                        box_type: BoxType::BlockBox(BoxProps {
                                            node_type: &node,
                                            properties: block.iter().cloned().collect(),
                                        }),
                                        children: vec![],
                                    },
                                    LayoutBox {
                                        box_type: BoxType::BlockBox(BoxProps {
                                            node_type: &node,
                                            properties: block.iter().cloned().collect(),
                                        }),
                                        children: vec![],
                                    }
                                ],
                            },
                            LayoutBox {
                                box_type: BoxType::InlineBox(BoxProps {
                                    node_type: &node,
                                    properties: inline.iter().cloned().collect(),
                                }),
                                children: vec![],
                            }
                        ]
                    },
                    LayoutBox {
                        box_type: BoxType::BlockBox(BoxProps {
                            node_type: &node,
                            properties: block.iter().cloned().collect(),
                        }),
                        children: vec![],
                    }
                ],
            }
        );
    }
}
