use crate::dom::AttrMap;
use combine::any;
use combine::parser::char::char;
use combine::parser::char::letter;
use combine::parser::char::newline;
use combine::parser::char::space;
use combine::parser::Parser;
use combine::sep_by;
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

fn attributes<Input>() -> impl Parser<Input, Output = AttrMap>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    sep_by::<Vec<(String, String)>, _, _, _>(
        attribute(),
        many::<String, _, _>(space().or(newline())),
    )
    .map(|attrs: Vec<(String, String)>| {
        let m: AttrMap = attrs.into_iter().collect();
        m
    })
}

fn open_tag<Input>() -> impl Parser<Input, Output = (String, AttrMap)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let tag_name = many1::<String, _, _>(letter());
    let tag_content = (
        tag_name,
        many::<String, _, _>(space().or(newline())),
        attributes(),
    );
    between(char('<'), char('>'), tag_content).map(|(tag_name, _, attrs)| (tag_name, attrs))
}

fn close_tag<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    todo!("implementation");
    many::<String, _, _>(letter()).map(|_| "".to_string())
}

mod test {
    use super::*;
    #[allow(unused_imports)]
    use crate::dom::AttrMap;

    #[test]
    fn test_parse_attriute() {
        assert_eq!(
            attribute().easy_parse("test=\"foobar\""),
            Ok((("test".to_string(), "foobar".to_string()), ""))
        )
    }

    #[test]
    fn test_parse_attributes() {
        let mut expected_map = AttrMap::new();
        expected_map.insert("test".to_string(), "foobar".to_string());
        expected_map.insert("abc".to_string(), "def".to_string());
        assert_eq!(
            attributes().easy_parse("test=\"foobar\" abc=\"def\""),
            Ok((expected_map, ""))
        );

        assert_eq!(attributes().easy_parse(""), Ok((AttrMap::new(), "")))
    }

    #[test]
    fn test_parse_open_tag() {
        println!("{:?}", open_tag().easy_parse("<hoge hogehoge>"));

        {
            assert_eq!(
                open_tag().easy_parse("<p>aaa"),
                Ok((("p".to_string(), AttrMap::new()), "aaa"))
            )
        }
        {
            let mut attributes = AttrMap::new();
            attributes.insert("id".to_string(), "test".to_string());
            assert_eq!(
                open_tag().easy_parse("<p id=\"test\">"),
                Ok((("p".to_string(), attributes), ""))
            )
        }
        {
            assert!(open_tag().easy_parse("<p id>").is_err());
        }
    }

    #[test]
    fn test_parse_close_tag() {
        let result = close_tag().easy_parse("</p>");
        assert_eq!(result, Ok(("p".to_string(), "")))
    }
}
