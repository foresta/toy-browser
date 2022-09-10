use combine::many;
use combine::many1;
use combine::parser::char::char;
use combine::parser::char::letter;
use combine::parser::char::newline;
use combine::parser::char::space;
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

fn whitespaces<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many::<String, _, _>(space().or(newline()))
}

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

mod tests {
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
}
