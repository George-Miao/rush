use pest::Span;

use crate::Rule;

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum Error<'src> {
    #[error("Parse error: {0}")]
    Pest(#[from] pest::error::Error<Rule>),
    #[error("Tree error: expect {expect:?}, found {found:?}, at {span:?}")]
    TreeError {
        expect: Vec<Rule>,
        found: Rule,
        span: Span<'src>,
    },
    #[error("Literal format error: expect {expect:?}, found {val:?}, at {span:?}")]
    LiteralError {
        expect: Rule,
        val: &'src str,
        span: Span<'src>,
    },
    #[error("Invalid input: {0}")]
    InputInvalid(&'src str),
}

pub type Result<'a, T> = std::result::Result<T, Error<'a>>;
