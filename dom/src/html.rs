use crate::blank;
use crate::dom::{AttrMap, Element, Node, NodeType, Text};
use combine::error::{ParseError, StreamError};
use combine::parser::char::{char, letter};
use combine::{attempt, between, choice, many, many1, parser, satisfy, sep_end_by, Parser, Stream};

/// `attribute` consumes `name="value"`.
// attribute := attribute_name S* "=" S* attribute_value
// attribute_name := alphabet+
// attribute_value := '"' attribute_inner_value '"'
// attribute_inner_value := (alphabet | digit | sign | " ")+
fn attribute<Input>() -> impl Parser<Input, Output = (String, String)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        // letter(): [a-zA-Z]
        // many1: +
        many1::<String, _, _>(letter()), // まずは属性の名前を何文字か読む
        // \s*
        blank(),
        // =
        char('='), // = を読む
        // \s*
        blank(),
        // "[!"]+"
        between(
            char('"'),
            char('"'),
            many1::<String, _, _>(satisfy(|c: char| c != '"')),
        ), // 引用符の間の、引用符を含まない文字を読む
    )
        .map(|v| (v.0, v.4))
}

/// `attributes` consumes `name1="value1" name2="value2" ... name="value"`
fn attributes<Input>() -> impl Parser<Input, Output = AttrMap>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    // 空白区切りで `attribute`を使いたい
    // https://docs.rs/combine/latest/combine/fn.sep_by.html
    //          ↓ `F` は変換先の型
    sep_end_by::<Vec<_>, _, _, _>(attribute(), blank()).map(|attrs| AttrMap::from_iter(attrs))
}

/// `open_tag` consumes `<tag_name attr_name="attr_value" ...>`.
fn open_tag<Input>() -> impl Parser<Input, Output = (String, AttrMap)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        char('<'),
        many1::<String, _, _>(letter()),
        blank(),
        attributes(),
        char('>'),
    )
        .map(|v| (v.1, v.3))
}

/// close_tag consumes `</tag_name>`.
fn close_tag<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        char('<'),
        char('/'),
        many1::<String, _, _>(letter()),
        char('>'),
    )
        .map(|v| v.2)
}

// `nodes_` (and `nodes`) tries to parse input as Element or Text.
fn nodes_<Input>() -> impl Parser<Input, Output = Vec<Box<Node>>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    // attempt
    // https://docs.rs/combine/latest/combine/fn.attempt.html

    // choice
    // https://docs.rs/combine/latest/combine/fn.choice.html

    // nodes := (node S*)*
    // node := element | text
    attempt(many(choice((attempt(element()), attempt(text())))))
}

/// `text` consumes input until `<` comes.
fn text<Input>() -> impl Parser<Input, Output = Box<Node>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(satisfy(|c: char| c != '<')).map(|t: String| Text::new(t.trim().to_string()))
}

/// `element` consumes `<tag_name attr_name="attr_value" ...>(children)</tag_name>`.
fn element<Input>() -> impl Parser<Input, Output = Box<Node>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (open_tag(), blank(), nodes(), blank(), close_tag()).and_then(
        |((open_tag_name, attributes), _, children, _, close_tag_name)| {
            if open_tag_name == close_tag_name {
                Ok(Element::new(open_tag_name, attributes, children))
            } else {
                Err(<Input::Error as combine::error::ParseError<
                    char,
                    Input::Range,
                    Input::Position,
                >>::StreamError::message_static_message(
                    "tag name of open tag and close tag mismatched",
                ))
            }
        },
    )
}

parser! {
    fn nodes[Input]()(Input) -> Vec<Box<Node>>
    where [Input: Stream<Token = char>]
    {
        nodes_().map(|nodes| nodes
            // text要素の前後をtrimしたものが空なら無視する
            .into_iter()
            .filter(|n| match &n.node_type {
                NodeType::Text(t) => !t.data.trim().is_empty(),
                _ => true,
            }).collect())
    }
}

pub fn parse(raw: &str) -> Box<Node> {
    let mut nodes = parse_raw(raw);
    if nodes.len() == 1 {
        nodes.pop().unwrap()
    } else {
        Element::new("html".to_string(), AttrMap::new(), nodes)
    }
}

pub fn parse_raw(raw: &str) -> Vec<Box<Node>> {
    let (nodes, _) = nodes().parse(raw).unwrap();
    nodes
}
#[cfg(test)]
mod tests {
    use crate::dom::Text;

    use super::*;

    // parsing tests of attributes
    #[test]
    fn test_parse_attribute() {
        assert_eq!(
            attribute().parse("test=\"foobar\""),
            Ok((("test".to_string(), "foobar".to_string()), ""))
        );

        assert_eq!(
            attribute().parse("test = \"foobar\""),
            Ok((("test".to_string(), "foobar".to_string()), ""))
        )
    }

    #[test]
    fn test_parse_attributes() {
        let mut expected_map = AttrMap::new();
        expected_map.insert("test".to_string(), "foobar".to_string());
        expected_map.insert("abc".to_string(), "def".to_string());
        assert_eq!(
            attributes().parse("test=\"foobar\" abc=\"def\""),
            Ok((expected_map, ""))
        );

        assert_eq!(attributes().parse(""), Ok((AttrMap::new(), "")))
    }

    #[test]
    fn test_parse_open_tag() {
        {
            assert_eq!(
                open_tag().parse("<p>aaaa"),
                Ok((("p".to_string(), AttrMap::new()), "aaaa"))
            );
        }
        {
            let mut attributes = AttrMap::new();
            attributes.insert("id".to_string(), "test".to_string());
            assert_eq!(
                open_tag().parse("<p id=\"test\">"),
                Ok((("p".to_string(), attributes), ""))
            )
        }
        {
            let mut attributes = AttrMap::new();
            attributes.insert("id".to_string(), "test".to_string());
            // TODO:
            assert_eq!(
                open_tag().parse("<p id=\"test\"  >"),
                Ok((("p".to_string(), attributes), ""))
            )
        }

        {
            let result = open_tag().parse("<p id=\"test\" class=\"sample\">");
            let mut attributes = AttrMap::new();
            attributes.insert("id".to_string(), "test".to_string());
            attributes.insert("class".to_string(), "sample".to_string());
            assert_eq!(result, Ok((("p".to_string(), attributes), "")));
        }

        {
            assert!(open_tag().parse("<p id>").is_err());
        }
    }

    // parsing tests of close tags
    #[test]
    fn test_parse_close_tag() {
        let result = close_tag().parse("</p>");
        assert_eq!(result, Ok(("p".to_string(), "")))
    }

    #[test]
    fn test_parse_element() {
        assert_eq!(
            element().parse("<p></p>"),
            Ok((Element::new("p".to_string(), AttrMap::new(), vec![]), ""))
        );

        assert_eq!(
            element().parse("<p>hello world</p>"),
            Ok((
                Element::new(
                    "p".to_string(),
                    AttrMap::new(),
                    vec![Text::new("hello world".to_string())]
                ),
                ""
            ))
        );

        assert_eq!(
            element().parse("<div>  <p>hello world</p>\n </div>"),
            Ok((
                Element::new(
                    "div".to_string(),
                    AttrMap::new(),
                    vec![Element::new(
                        "p".to_string(),
                        AttrMap::new(),
                        vec![Text::new("hello world".to_string())]
                    )],
                ),
                ""
            ))
        );

        assert!(element().parse("<p>hello world</div>").is_err());
    }

    #[test]
    fn test_parse_text() {
        {
            assert_eq!(
                text().parse("Hello World"),
                Ok((Text::new("Hello World".to_string()), ""))
            );
        }
        {
            assert_eq!(
                text().parse("Hello World<"),
                Ok((Text::new("Hello World".to_string()), "<"))
            );
        }
    }
}
