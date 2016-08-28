use std::fmt;
use std::num::Wrapping;
use std::collections::HashMap;
use self::AstNode::*;

pub type Cell = Wrapping<i8>;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Position{
    pub start: usize,
    pub end: usize,
}
