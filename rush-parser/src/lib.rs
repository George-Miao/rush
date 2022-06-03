#![cfg_attr(test, feature(assert_matches))]

use pest::Parser;
pub mod ast;
mod error;
mod impl_ast;

#[cfg(test)]
mod test;

pub use error::*;

use crate::ast::Tree;

#[derive(pest_derive::Parser)]
#[grammar = "pest/rush.pest"]
pub struct RushParser;

pub fn parse(input: &str) -> Result<Tree<'_>> {
    let res = RushParser::parse(Rule::main, input)?.next().unwrap();
    let tree = Tree::try_from(res)?;

    Ok(tree)
}
