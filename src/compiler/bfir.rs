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

fn fmt_with_indent(instr: &AstNode, indent: i32, f: &mut fmt::Formatter){
    for _ in 0..indent{
        let _ = write!(f, " ");
    }
    match instr{
        &Loop {body: ref loop_body, position, ..} => {
            let _ = write!(f, "Loop position: {:?}", position);
            for loop_instr in loop_body{
                let _ = write!(f, "\n");
                fmt_with_indent(loop_instr, indent + 1, f);
            }
        }
        instr => {
            let _ = write!(f, "{:?}", instr);
        }
    }
}

impl fmt::Display for AstNode{
    fn fmt(&self, f: *mut fmt::Formatter) -> fmt::Result{
        fmt_with_indent(self, 0, f);
        Ok(())
    }
}

pub fn get_position(instr: &AstNode) -> Option<Position>{
    match *instr{
        _ {position, ..} => position,
        //Fixme
    }
}

#[derive(Debug)]
pub struct ParseError{
    pub message: String,
    pub position: Position,
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

