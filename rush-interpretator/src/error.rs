#![allow(clippy::module_name_repetitions)]

use std::process::Output;

use thiserror::Error;

use crate::Ref;

#[derive(Error, Debug)]
pub enum Error<'src> {
    #[error("{0}")]
    Command(#[from] CommandError),
    #[error("{0}")]
    Runtime(#[from] RuntimeError),
    #[error("{0}")]
    Parse(parser::Error<'src>),
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Identifier `{0}` not found")]
    IdentNotFound(String),
    #[error("Type of `{ident}` mismatched: expect `{expected}`, found `{found}`")]
    TypeError {
        ident: String,
        expected: String,
        found: String,
    },
    #[error("Expected {expected} arguments to call `{ident}`, found {found}")]
    ArgumentError {
        ident: String,
        expected: usize,
        found: usize,
    },
    #[error("Ref not found: `{0}`")]
    NullRefError(Ref),
    #[error("Max recursion depth exceeded")]
    MaxRecursionExceeded,
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Command execution error: {0}")]
    Command(#[from] std::io::Error),
    #[error("Command output is not UTF-8")]
    CodingError(#[from] std::string::FromUtf8Error),
}

pub type Result<'a, T> = std::result::Result<T, Error<'a>>;
pub type RuntimeResult<T> = std::result::Result<T, RuntimeError>;
pub type CommandResult<O = Output> = std::result::Result<O, CommandError>;
