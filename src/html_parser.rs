use combine::parser::char::char;
use combine::parser::char::letter;
use combine::parser::char::newline;
use combine::parser::char::space;
use combine::parser::Parser;
#[allow(unused_imports)]
use combine::EasyParser;
use combine::ParseError;
use combine::Stream;
use combine::{between, many, many1, satisfy};

fn attribute<Input>() -> impl Parser<Input, Output = (String, String)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        many1::<String, _, _>(letter()), // read attribute name any letters
        many::<String, _, _>(space().or(newline())), // skip read space and newline
        char('='),                       // read '='
        many::<String, _, _>(space().or(newline())), // skip read space and newline
        between(
            char('"'),
            char('"'),
            many1::<String, _, _>(satisfy(|c: char| c != '"')),
        ), // read character expecting quates
    )
        .map(|v| (v.0, v.4))
}

mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_parse_attriute() {
        assert_eq!(
            attribute().easy_parse("test=\"foobar\""),
            Ok((("test".to_string(), "foobar".to_string()), ""))
        )
    }
}
