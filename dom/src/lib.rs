use combine::error::ParseError;
use combine::parser::char::{newline, space};
use combine::{many, Parser, Stream};

pub mod css;
pub mod dom;
pub mod html;
pub mod layout;
pub mod style;

fn blank<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    // 空白と改行を読み飛ばす
    many::<String, _, _>(space().or(newline())).map(|_| ())
}
