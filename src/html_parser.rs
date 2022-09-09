use crate::dom::Node;
#[allow(unused_imports)]
use crate::dom::{AttrMap, Element, Text};
use combine::attempt;
use combine::choice;
use combine::error::StreamError;
use combine::parser;
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

#[allow(dead_code)]
fn attribute<Input>() -> impl Parser<Input, Output = (String, String)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        many1::<String, _, _>(letter().or(char('-'))), // read attribute name any letters
        many::<String, _, _>(space().or(newline())),   // skip read space and newline
        char('='),                                     // read '='
        many::<String, _, _>(space().or(newline())),   // skip read space and newline
        between(
            char('"'),
            char('"'),
            many1::<String, _, _>(satisfy(|c: char| c != '"')),
        ), // read character expecting quates
    )
        .map(|v| (v.0, v.4))
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

parser! {
    fn contents[Input]()(Input) -> Vec<Box<Node>>
        where [Input: Stream<Token = char>]
    {
        contents_()
    }
}

fn contents_<Input>() -> impl Parser<Input, Output = Vec<Box<Node>>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    attempt(many(choice((attempt(element()), attempt(text())))))
}

fn text<Input>() -> impl Parser<Input, Output = Box<Node>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(satisfy(|c: char| c != '<')).map(|t| Text::new(t))
}

#[allow(dead_code)]
fn element<Input>() -> impl Parser<Input, Output = Box<Node>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (open_tag(), contents(), close_tag()).and_then(
        |((open_tag_name, attributes), children, close_tag_name)| {
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

mod test {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::dom::{AttrMap, Element, Text};

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

    #[test]
    fn test_parse_element() {
        println!(
            "{:?}",
            element().easy_parse("<div><p>hello world</p><span class=\"red\">hoge</span></div>")
        );

        assert_eq!(
            element().easy_parse("<p></p>"),
            Ok((Element::new("p".to_string(), AttrMap::new(), vec![]), ""))
        );

        assert_eq!(
            element().easy_parse("<p>hello world</p>"),
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
            element().easy_parse("<div><p>hello world</p></div>"),
            Ok((
                Element::new(
                    "div".to_string(),
                    AttrMap::new(),
                    vec![Element::new(
                        "p".to_string(),
                        AttrMap::new(),
                        vec![Text::new("hello world".to_string())]
                    )]
                ),
                ""
            ))
        );

        assert!(element().easy_parse("<p>hello world</div>").is_err());
    }
}
