use std::collections::HashMap;
use std::num::Wrapping;
use quickcheck::quickcheck;
use bfir::{parse, Position};
use bfir::{AstNode, Cell};
use bfir::AstNode::*;
use diagnostics::Warning;
use bounds::MAX_CELL_INDEX;
use bounds::highest_cell_index;

