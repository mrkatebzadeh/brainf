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

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum AstNode{
    Increment{
        amount: Cell,
        offset: isize,
        position: Option<Position>,
    },

    PointerIncrement{
        amount: isize,
        position: Option<Position>,
    },

    Read{
        position: Option<Position>,
    },

    Write{
        position: Option<Position>,
    },

    Loop{
        body: Vec<AstNode>,
        position: Option<Position>,
    },
    
}

pub trait Combine<T>{
    fn combine(&self, T) -> T;
}
impl Combine<Option<Position>> for Option<Position>{
    fn combine(&self, other: Self) -> Self{
        match(*self, other){
            (Some(pos1), Some(pos2)) => {
                let (first_pos, second_pos) = if pos1.start <= pos2.start{
                    (pos1, pos2)
                } else{
                    (pos2, pos1)
                };

                if first_pos.end + 1 >= second_pos.start{
                    Some(Position{
                        start: first_pos.start,
                        end: second_pos.end,
                    })
                } else{
                    Some(pos2)
                }
            }
            _ => None,
        }
    }
}

