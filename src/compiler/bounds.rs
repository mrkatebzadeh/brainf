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

impl Add for SaturatingInt{
    type Output = SaturatingInt;
    fn add(self, rhs: SaturatingInt) -> SaturatingInt{
        if let (&SaturatingInt::Number(x), &SaturatingInt::Number(y)) = (&self, &rhs){
            SaturatingInt::Number(x + y)
        } else{
            SaturatingInt::Max
        }
    }
}

impl Ord for SaturatingInt{
    fn cmp(&self, other: &SaturatingInt) -> Ordering{
        match(self, other){
            (&SaturatingInt::Max, &SaturatingInt::Max) => Ordering::Equal,
            (&SaturatingInt::Number(_), &SaturatingInt::Max) => Ordering::Less,
            (&SaturatingInt::Max, &SaturatingInt::Number(_)) => Ordering::Greater,
            (&SaturatingInt::Number(x), &SaturatingInt::Number(y)) => x.cmp(&y),
        }
    }
}

fn overall_movement(instrs: &[AstNode]) -> (SaturatingInt, SaturatingInt){
    let mut net_movement = SaturatingInt::Number(0);
    let mut max_index = SaturatingInt::Number(0);

    for (instr_highest_offset, instr_net_movement) in instrs.iter().map(movement){
        max_index = max(net_movement,
                        max(net_movement + instr_highest_offset, max_index));
        net_movement = net_movement + instr_net_movement;
    }
    (max_index, net_movement)
}

fn movement(instr: &AstNode) -> (SaturatingInt, SaturatingInt){
    match *instr{
        PointerIncrement{amount, ..} => {
            if amount < 0{
                (SaturatingInt::Number(0),
                SaturatingInt::Number(amount as i64))
            } else{
                (SaturatingInt::Number(amount as i64),
                SaturatingInt::Number(amount as i64))
            }
        }

        Increment{offset, ..} => {
            (SaturatingInt::Number(offset as i64),
            SaturatingInt::Number(0))
        }

        Set{offset, ..} => {
            (SaturatingInt::Number(offset as i64),
            SaturatingInt::Number(0))
        }

        MultiplyMove{ref changes, ..} => {
            let mut highest_affected = 0;
            for cell in changes.keys(){
                if *cell > highest_affected {
                    highest_affected = *cell;
                }
            }
            (SaturatingInt::Number(highest_affected as i64),
            SaturatingInt::Number(0))
        }
        Loop{ref body, ..} => {
            let (max_in_body, net_in_body) = overall_movement(body);
            match net_in_body{
                SaturatingInt::Number(net_loop_movement) => {
                    if net_loop_movement == 0{
                        (max_in_body, SaturatingInt::Number(0))
                    } else if net_loop_movement < 0{
                        (max_in_body, SaturatingInt::Number(0))
                    } else{
                        (SaturatingInt::Max, SaturatingInt::Number(0))
                    }
                }
                SaturatingInt::Max => {
                    (SaturatingInt::Max, SaturatingInt::Max)
                }
            }
        }
        Read{..} | Write{..} => (SaturatingInt::Number(0), SaturatingInt::Number(0)),
    }
}
