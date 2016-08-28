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

impl<'a> ExecutionState<'a>{
    pub fn initial(instrs: &[AstNode]) -> Self{
        ExecutionState{
            start_instr: Node,
            cells: vec![Wrapping(0); highest_cell_index(instrs) + 1],
            cell_ptr: 0,
            outputs: vec![],
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome{
    Completed(u64),
    ReachedRuntimeValue,
    RuntimeError(Warning),
    OutOfSteps,
}

pub const MAX_STEPS: u64 = 10000000;

