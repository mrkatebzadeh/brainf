use quickcheck::quickcheck;
use std::collections::HashMap;
use std::num::Wrapping;
use std::ops::Add;
use std::cmp::{Ord, Ordering, max};
use bfir::AstNode;
use bfir::AstNode::*;
use bfir::{parse, Position};

pub const MAX_CELL_INDEX: usize = 99999;

pub fn highest_cell_index(instrs: &[AstNode]) -> usize{
    let (highest_index, _) = overall_movement(instrs);
    match highest_index{
        SaturatingInt::Number(x) => {
            if x > MAX_CELL_INDEX as i64{
                MAX_CELL_INDEX
            } else{
                x as usize
            }
        }
        SaturatingInt::Max => MAX_CELL_INDEX,
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum SaturatingInt{
    Number(i64),
    Max,
}
