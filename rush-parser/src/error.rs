use pest::Span;

use crate::Rule;

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum Error<'a> {
    #[error("Parse error: {0}")]
    Pest(#[from] pest::error::Error<Rule>),
    #[error("Logic error: expect {expect:?}, found {found:?}, at {span:?}")]
    TreeError {
        expect: Rule,
        found: Rule,
        span: Span<'a>,
    },
    #[error("Literal format error: expect {expect:?}, found {val:?}, at {span:?}")]
    LiteralError {
        expect: Rule,
        val: &'a str,
        span: Span<'a>,
    },
    #[error("Invalid input: {0}")]
    InputInvalid(&'a str),
}

pub type Result<'a, T> = std::result::Result<T, Error<'a>>;
