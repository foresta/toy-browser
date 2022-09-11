use combine::attempt;
use combine::choice;
use combine::error::StreamError;
use combine::many;
use combine::many1;
use combine::parser::char::char;
use combine::parser::char::letter;
use combine::parser::char::newline;
use combine::parser::char::space;
use combine::parser::char::string;
use combine::sep_end_by;
use combine::ParseError;
use combine::Parser;
use combine::Stream;

pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug, PartialEq)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

// TODO: Support combinator selector
//       like `div p { ... }`, div > p { ... }, div * { ... }, div + p { ... }
pub type Selector = SimpleSelector;

// https://www.w3.org/TR/selectors-3/
// UniversalSelector: *
// TypeSelector: h1, div, span, ...etc
// AttributeSelector: h1[title], a[href="https://example.com"], a[href~="example"]
//  op:
//      support: = (Eq), ~= (Contain),
//      non support: |=, ^=, $=, *=
// ClassSelector: .class-name
// IDSelector: #id_name
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
    IdSelector {
        id_name: String,
    },
}

#[derive(Debug, PartialEq)]
pub enum AttributeSelectorOp {
    Eq,      // =
    Contain, // ~=
}

#[derive(Debug, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: CSSValue,
    // TODO: support !important
}

#[derive(Debug, PartialEq, Clone)]
pub enum CSSValue {
    Keyword(String),
}

#[allow(dead_code)]
fn whitespaces<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many::<String, _, _>(space().or(newline()))
}

#[allow(dead_code)]
fn declaration<Input>() -> impl Parser<Input, Output = Declaration>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let name = many1(letter()).skip(whitespaces());
    let seperator = char(':').skip(whitespaces());
    let value = many1(letter()).map(|v| CSSValue::Keyword(v));
    (name, seperator, value).map(|(name, _, value)| Declaration {
        name: name,
        value: value,
    })
}

#[allow(dead_code)]
fn declarations<Input>() -> impl Parser<Input, Output = Vec<Declaration>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    sep_end_by(
        declaration().skip(whitespaces()),
        char(';').skip(whitespaces()),
    )
}

#[allow(dead_code)]
fn selectors<Input>() -> impl Parser<Input, Output = Vec<Selector>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    char(' ').map(|_| vec![SimpleSelector::UniversalSelector])
}

#[allow(dead_code)]
fn simple_selector<Input>() -> impl Parser<Input, Output = Selector>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    choice((
        universal_selector(),
        id_selector(),
        class_selector(),
        attribute_selector(),
        type_selector(),
    ))
}

fn universal_selector<Input>() -> impl Parser<Input, Output = Selector>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    char('*').map(|_| SimpleSelector::UniversalSelector)
}

fn class_selector<Input>() -> impl Parser<Input, Output = Selector>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (char('.'), many1(letter())).map(|(_, class_name)| SimpleSelector::ClassSelector {
        class_name: class_name,
    })
}

fn id_selector<Input>() -> impl Parser<Input, Output = Selector>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (char('#'), many1(letter())).map(|(_, id_name)| SimpleSelector::IdSelector { id_name: id_name })
}

fn type_selector<Input>() -> impl Parser<Input, Output = Selector>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(letter()).map(|v| SimpleSelector::TypeSelector { tag_name: v })
}

fn attribute<Input>() -> impl Parser<Input, Output = (String, AttributeSelectorOp, String)>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    // parse: attr=value
    (
        many1(letter()),
        choice((string("="), string("~="))),
        many1(letter()),
    )
        .and_then(|(attr, op_symbol, value)| {
            let op = match op_symbol {
                "=" => AttributeSelectorOp::Eq,
                "~=" => AttributeSelectorOp::Contain,
                _ => {
                    return Err(<Input::Error as combine::error::ParseError<
                        char,
                        Input::Range,
                        Input::Position,
                    >>::StreamError::message_static_message(
                        "Invalid attribute selector op",
                    ))
                }
            };

            Ok((attr, op, value))
        })
}

fn attribute_selector<Input>() -> impl Parser<Input, Output = Selector>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    // parse: tag_name[attr=value]
    attempt((many1(letter()), char('['), attribute(), char(']'))).map(
        |(tag_name, _, (attr, op, value), _)| SimpleSelector::AttributeSelector {
            tag_name: tag_name,
            attribute: attr,
            op: op,
            value: value,
        },
    )
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

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
        )
    }

    #[test]
    fn test_selectors() {
        assert_eq!(
            selectors().parse("test[foo=bar], a"),
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
        )
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
                    tag_name: "test".to_string()
                },
                ""
            ))
        );

        assert_eq!(
            simple_selector().parse("test[foo=bar]"),
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
                    class_name: "test".to_string()
                },
                ""
            ))
        );

        assert_eq!(
            simple_selector().parse("#test"),
            Ok((
                SimpleSelector::IdSelector {
                    id_name: "test".to_string()
                },
                ""
            ))
        );
    }
}
