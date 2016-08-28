use std::collections::HashMap;
use std::num::Wrapping;
use quickcheck::quickcheck;
use bfir::{parse, Position};
use bfir::{AstNode, Cell};
use bfir::AstNode::*;
use diagnostics::Warning;
use bounds::MAX_CELL_INDEX;
use bounds::highest_cell_index;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionState<'a>{
    pub start_instr: Option<&'a AstNode>,
    pub cells: Vec<Cell>,
    pub cell_ptr: isize,
    pub outputs: Vec<i8>,
}
