use crate::dom::{Node, NodeType};
use anyhow::Result;
use combine::{
    choice,
    error::StreamError,
    many1, optional,
    parser::char::{self, char, letter, spaces, string},
    sep_by, sep_end_by, ParseError, Parser, Stream,
};

/// `Stylesheet` represents a single stylesheet.
/// It consists of multiple rules, which are called "rule-list" in the standard (https://www.w3.org/TR/css-syntax-3/).
#[derive(Debug, PartialEq)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

impl Stylesheet {
    pub fn new(rules: Vec<Rule>) -> Self {
        Stylesheet { rules: rules }
    }
}

/// `Rule` represents a single CSS rule.
#[derive(Debug, PartialEq)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

impl Rule {
    pub fn matches(&self, node: &Box<Node>) -> bool {
        self.selectors.iter().any(|s| s.matches(node))
    }
}

/// NOTE: This is not compliant to the standard for simplicity.
///
/// In the standard, *a selector* is *a chain* of one or more sequences of simple selectors separated by combinators,
/// where a sequence of simple selectors is a chain of simple selectors that are not separated by a combinator.
/// Hence `Selector` is in fact something like `Vec<Vec<SimpleSelector>>`.
pub type Selector = SimpleSelector;

/// `SimpleSelector` represents a simple selector defined in the following standard:
/// https://www.w3.org/TR/selectors-3/#selector-syntax
#[derive(Debug, PartialEq)]
pub enum SimpleSelector {
    UniversalSelector,
    TypeSelector {
        tag_name: String,
    },
    AttributeSelector {
        tag_name: String,
        op: AttributeSelectorOp,
        attribute: String,
        value: String,
    },
    ClassSelector {
        class_name: String,
    },
    // TODO (enhancement): support multiple attribute selectors like `a[href=bar][ping=foo]`
    // TODO (enhancement): support more attribute selectors
}

impl SimpleSelector {
    fn matches(&self, node: &Box<Node>) -> bool {
        match self {
            SimpleSelector::UniversalSelector => true,
            SimpleSelector::TypeSelector { tag_name } => match &node.node_type {
                NodeType::Element(node) => node.tag_name == *tag_name,
                _ => false,
            },
            // class="hoge fuga"
            // p [class ~= hoge] {}
            SimpleSelector::AttributeSelector {
                tag_name,
                op,
                attribute,
                value,
            } => {
                let NodeType::Element(e) = &node.node_type else {
                    return false;
                };
                if e.tag_name != *tag_name {
                    return false;
                }
                match op {
                    AttributeSelectorOp::Eq => e.attributes.get(attribute) == Some(value),
                    AttributeSelectorOp::Contain => e
                        .attributes
                        .get(attribute)
                        .map_or(false, |v| v.split_whitespace().any(|v| v == value)),
                }
            }
            SimpleSelector::ClassSelector { class_name } => {
                let NodeType::Element(e) = &node.node_type else {
                    return false;
                };
                e.attributes.get("class") == Some(class_name)
            }
        }
    }
}

/// `AttributeSelectorOp` is an operator which is allowed to use.
/// See https://www.w3.org/TR/selectors-3/#attribute-selectors to check the full list of available operators.
#[derive(Debug, PartialEq)]
pub enum AttributeSelectorOp {
    Eq,      // =
    Contain, // ~=
}

/// `Declaration` represents a CSS declaration defined at [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/#declaration)
///
/// Declarations are further categorized into the followings:
/// - descriptors, which are mostly used in "at-rules" like `@foo (bar: piyo)` https://www.w3.org/Style/CSS/all-descriptors.en.html
/// - properties, which are mostly used in "qualified rules" like `.foo {bar: piyo}` https://www.w3.org/Style/CSS/all-descriptors.en.html
///
/// For simplicity, we handle two types of declarations together.
#[derive(Debug, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: CSSValue,
    // TODO (enhancement): add a field for `!important`
}

/// `CSSValue` represents some of *component value types* defined at [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/#component-types).
#[derive(Debug, PartialEq, Clone)]
pub enum CSSValue {
    Keyword(String),
}

pub fn parse(raw: &str) -> Result<Stylesheet> {
    rules()
        .parse(raw)
        .map(|(rules, _)| Stylesheet::new(rules))
        .map_err(|e| e.into())
}

fn rules<Input>() -> impl Parser<Input, Output = Vec<Rule>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    // TODO: 末尾に空白とか改行があるとエラーになる
    (spaces(), sep_by(rule(), spaces()), spaces()).map(|(_, rules, _)| rules)
}

fn rule<Input>() -> impl Parser<Input, Output = Rule>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        selectors(),
        spaces(),
        char('{'),
        spaces(),
        declarations(),
        spaces(),
        char('}'),
    )
        .map(|(selectors, _, _, _, declarations, _, _)| Rule {
            selectors,
            declarations,
        })
}

fn selectors<Input>() -> impl Parser<Input, Output = Vec<Selector>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    sep_by(
        (simple_selector(), spaces()).map(|(s, _)| s),
        (char(','), spaces()),
    )
}

fn universal_selector<Input>() -> impl Parser<Input, Output = SimpleSelector>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    char('*').map(|_| SimpleSelector::UniversalSelector)
}

fn class_selector<Input>() -> impl Parser<Input, Output = SimpleSelector>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (char('.'), many1(letter())).map(|(_, class_name)| SimpleSelector::ClassSelector { class_name })
}

fn selector_op<Input>() -> impl Parser<Input, Output = Result<AttributeSelectorOp, <Input::Error as combine::error::ParseError<
                    char,
                    Input::Range,
                    Input::Position,
                >>::StreamError>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice((string("="), string("~="))).map(|op| match op {
        "=" => Ok(AttributeSelectorOp::Eq),
        "~=" => Ok(AttributeSelectorOp::Contain),
        _ => Err(<Input::Error as combine::error::ParseError<
            char,
            Input::Range,
            Input::Position,
        >>::StreamError::message_static_message(
            "invalid attribute selector op",
        )),
    })
}

fn type_or_attribute_selector<Input>() -> impl Parser<Input, Output = SimpleSelector>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        many1(letter()),
        spaces(),
        optional((
            char('['),
            spaces(),
            many1(letter()),
            selector_op(),
            many1(letter()),
            char(']'),
        )),
    )
        .and_then(|(tag_name, _, opt)| {
            let Some((_, _, attribute, op, value, _)) = opt else {
                return Ok(SimpleSelector::TypeSelector { tag_name });
            };
            let Ok(op) = op else {
                return Err(<Input::Error as combine::error::ParseError<
                    char,
                    Input::Range,
                    Input::Position,
                >>::StreamError::message_static_message(
                    "invalid attribute selector"
                ));
            };
            Ok(SimpleSelector::AttributeSelector {
                tag_name,
                op,
                attribute,
                value,
            })
        })
}

fn simple_selector<Input>() -> impl Parser<Input, Output = SimpleSelector>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice((
        universal_selector(),
        class_selector(),
        type_or_attribute_selector(),
    ))
}

fn declarations<Input>() -> impl Parser<Input, Output = Vec<Declaration>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    sep_end_by(
        (declaration(), spaces()).map(|(v, _)| v),
        (char(';'), spaces()).map(|(v, _)| v),
    )
}

fn declaration<Input>() -> impl Parser<Input, Output = Declaration>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (many1(letter()), spaces(), char(':'), spaces(), css_value()).map(|v| Declaration {
        name: v.0,
        value: v.4,
    })
}

fn css_value<Input>() -> impl Parser<Input, Output = CSSValue>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(letter()).map(|v| CSSValue::Keyword(v))
}

#[cfg(test)]
mod tests {
    use crate::dom::Element;

    use super::*;

    #[test]
    fn test_stylesheet() {
        assert_eq!(
            rules().parse("test [foo=bar] { aa: bb; cc: dd } rule { ee: dd;  }"),
            Ok((
                vec![
                    Rule {
                        selectors: vec![SimpleSelector::AttributeSelector {
                            tag_name: "test".to_string(),
                            attribute: "foo".to_string(),
                            op: AttributeSelectorOp::Eq,
                            value: "bar".to_string()
                        }],
                        declarations: vec![
                            Declaration {
                                name: "aa".to_string(),
                                value: CSSValue::Keyword("bb".to_string())
                            },
                            Declaration {
                                name: "cc".to_string(),
                                value: CSSValue::Keyword("dd".to_string()),
                            }
                        ]
                    },
                    Rule {
                        selectors: vec![SimpleSelector::TypeSelector {
                            tag_name: "rule".to_string(),
                        }],
                        declarations: vec![Declaration {
                            name: "ee".to_string(),
                            value: CSSValue::Keyword("dd".to_string())
                        }]
                    },
                ],
                ""
            ))
        );
    }

    #[test]
    fn test_rule() {
        assert_eq!(
            rule().parse("test [foo=bar] {}"),
            Ok((
                Rule {
                    selectors: vec![SimpleSelector::AttributeSelector {
                        tag_name: "test".to_string(),
                        attribute: "foo".to_string(),
                        op: AttributeSelectorOp::Eq,
                        value: "bar".to_string()
                    }],
                    declarations: vec![]
                },
                ""
            ))
        );

        assert_eq!(
            rule().parse("test [foo=bar], testtest[piyo~=guoo] {}"),
            Ok((
                Rule {
                    selectors: vec![
                        SimpleSelector::AttributeSelector {
                            tag_name: "test".to_string(),
                            attribute: "foo".to_string(),
                            op: AttributeSelectorOp::Eq,
                            value: "bar".to_string()
                        },
                        SimpleSelector::AttributeSelector {
                            tag_name: "testtest".to_string(),
                            attribute: "piyo".to_string(),
                            op: AttributeSelectorOp::Contain,
                            value: "guoo".to_string()
                        }
                    ],
                    declarations: vec![]
                },
                ""
            ))
        );

        assert_eq!(
            rule().parse("test [foo=bar] { aa: bb; cc: dd; }"),
            Ok((
                Rule {
                    selectors: vec![SimpleSelector::AttributeSelector {
                        tag_name: "test".to_string(),
                        attribute: "foo".to_string(),
                        op: AttributeSelectorOp::Eq,
                        value: "bar".to_string()
                    }],
                    declarations: vec![
                        Declaration {
                            name: "aa".to_string(),
                            value: CSSValue::Keyword("bb".to_string())
                        },
                        Declaration {
                            name: "cc".to_string(),
                            value: CSSValue::Keyword("dd".to_string()),
                        }
                    ]
                },
                ""
            ))
        );
    }

    #[test]
    fn test_declarations() {
        assert_eq!(
            declarations().parse("foo: bar; piyo: piyopiyo;"),
            Ok((
                vec![
                    Declaration {
                        name: "foo".to_string(),
                        value: CSSValue::Keyword("bar".to_string())
                    },
                    Declaration {
                        name: "piyo".to_string(),
                        value: CSSValue::Keyword("piyopiyo".to_string())
                    }
                ],
                ""
            ))
        );
    }

    #[test]
    fn test_selectors() {
        assert_eq!(
            selectors().parse("test [foo=bar], a"),
            Ok((
                vec![
                    SimpleSelector::AttributeSelector {
                        tag_name: "test".to_string(),
                        attribute: "foo".to_string(),
                        op: AttributeSelectorOp::Eq,
                        value: "bar".to_string()
                    },
                    SimpleSelector::TypeSelector {
                        tag_name: "a".to_string(),
                    }
                ],
                ""
            ))
        );
    }

    #[test]
    fn test_simple_selector() {
        assert_eq!(
            simple_selector().parse("*"),
            Ok((SimpleSelector::UniversalSelector, ""))
        );

        assert_eq!(
            simple_selector().parse("test"),
            Ok((
                SimpleSelector::TypeSelector {
                    tag_name: "test".to_string(),
                },
                ""
            ))
        );

        assert_eq!(
            simple_selector().parse("test [foo=bar]"),
            Ok((
                SimpleSelector::AttributeSelector {
                    tag_name: "test".to_string(),
                    attribute: "foo".to_string(),
                    op: AttributeSelectorOp::Eq,
                    value: "bar".to_string()
                },
                ""
            ))
        );

        assert_eq!(
            simple_selector().parse(".test"),
            Ok((
                SimpleSelector::ClassSelector {
                    class_name: "test".to_string(),
                },
                ""
            ))
        );
    }

    #[test]
    fn test_declaration() {
        assert_eq!(
            declaration().parse("keykey:piyo"),
            Ok((
                Declaration {
                    name: "keykey".to_string(),
                    value: CSSValue::Keyword("piyo".to_string()),
                },
                ""
            ))
        );

        assert_eq!(
            declaration().parse("keyabc : piyo "),
            Ok((
                Declaration {
                    name: "keyabc".to_string(),
                    value: CSSValue::Keyword("piyo".to_string()),
                },
                " "
            ))
        );

        assert_eq!(
            declaration().parse("keyhello : piyo "),
            Ok((
                Declaration {
                    name: "keyhello".to_string(),
                    value: CSSValue::Keyword("piyo".to_string()),
                },
                " "
            ))
        );

        assert!(declaration().parse("aaaaa").is_err())
    }

    #[test]
    fn test_universal_selector_behaviour() {
        let e = &Element::new(
            "p".to_string(),
            [
                ("id".to_string(), "test".to_string()),
                ("class".to_string(), "testclass".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            vec![],
        );
        assert_eq!(SimpleSelector::UniversalSelector.matches(e), true);
    }

    #[test]
    fn test_type_selector_behaviour() {
        let e = &Element::new(
            "p".to_string(),
            [
                ("id".to_string(), "test".to_string()),
                ("class".to_string(), "testclass".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            vec![],
        );

        assert_eq!(
            (SimpleSelector::TypeSelector {
                tag_name: "p".into(),
            })
            .matches(e),
            true
        );

        assert_eq!(
            (SimpleSelector::TypeSelector {
                tag_name: "invalid".into(),
            })
            .matches(e),
            false
        );
    }

    #[test]
    fn test_attribute_selector_behaviour() {
        let e = &Element::new(
            "p".to_string(),
            [
                ("id".to_string(), "test".to_string()),
                ("class".to_string(), "testclass".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            vec![],
        );

        assert_eq!(
            (SimpleSelector::AttributeSelector {
                tag_name: "p".into(),
                attribute: "id".into(),
                value: "test".into(),
                op: AttributeSelectorOp::Eq,
            })
            .matches(e),
            true
        );

        assert_eq!(
            (SimpleSelector::AttributeSelector {
                tag_name: "p".into(),
                attribute: "id".into(),
                value: "invalid".into(),
                op: AttributeSelectorOp::Eq,
            })
            .matches(e),
            false
        );

        assert_eq!(
            (SimpleSelector::AttributeSelector {
                tag_name: "p".into(),
                attribute: "invalid".into(),
                value: "test".into(),
                op: AttributeSelectorOp::Eq,
            })
            .matches(e),
            false
        );

        assert_eq!(
            (SimpleSelector::AttributeSelector {
                tag_name: "invalid".into(),
                attribute: "id".into(),
                value: "test".into(),
                op: AttributeSelectorOp::Eq,
            })
            .matches(e),
            false
        );
    }

    #[test]
    fn test_class_selector_behaviour() {
        let e = &Element::new(
            "p".to_string(),
            [
                ("id".to_string(), "test".to_string()),
                ("class".to_string(), "testclass".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
            vec![],
        );

        assert_eq!(
            (SimpleSelector::ClassSelector {
                class_name: "testclass".into(),
            })
            .matches(e),
            true
        );

        assert_eq!(
            (SimpleSelector::ClassSelector {
                class_name: "invalid".into(),
            })
            .matches(e),
            false
        );
    }
}
