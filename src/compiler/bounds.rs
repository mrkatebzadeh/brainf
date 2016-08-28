use quickcheck::quickcheck;
use std::collections::HashMap;
use std::num::Wrapping;
use std::ops::Add;
use std::cmp::{Ord, Ordering, max};
use bfir::AstNode;
use bfir::AstNode::*;
use bfir::{parse, Position};

pub const MAX_CELL_INDEX: usize = 99999;
