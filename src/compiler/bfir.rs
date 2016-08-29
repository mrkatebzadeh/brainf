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
    Set{
        amount: Cell,
        offset: isize,
        position: Option<Position>,
    },

    MultiplyMove{
        changes: HashMap<isize, Cell>,
        position: Option<Position>,
    },

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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        fmt_with_indent(self, 0, f);
        Ok(())
    }
}

pub fn get_position(instr: &AstNode) -> Option<Position>{
    match *instr{
        Increment { position, .. } => position,
        PointerIncrement { position, .. } => position,
        Read { position } => position,
        Write { position } => position,
        Loop { position, .. } => position,
        Set { position, .. } => position,
        MultiplyMove { position, .. } => position,
    }

}

#[derive(Debug)]
pub struct ParseError{
    pub message: String,
    pub position: Position,
}

pub fn parse(source: &str) -> Result<Vec<AstNode>, ParseError>{
    let mut instructions = vec![];
    let mut stack = vec![];

    for(index, c) in source.chars().enumerate(){
        match c{
            '+' => {
                instructions.push(Increment{
                    amount: Wrapping(1),
                    offset: 0,
                    position: Some(Position{
                        start: index,
                        end: index,
                    }),
                })
            }
            '-' => {
                instructions.push(Increment{
                    amount: Wrapping(-1),
                    offset: 0,
                    position: Some(Position{
                        start: index,
                        end: index,
                    }),
                })
            }
            '>' => {
                instructions.push(PointerIncrement{
                    amount: 1,
                    position: Some(Position{
                        start: index,
                        end: index,
                    }),
                })
            }
            '<' => {
                instructions.push(PointerIncrement{
                    amount: -1,
                    position: Some(Position{
                        start: index,
                        end: index,
                    }),
                })
            }
            '.' => {
                instructions.push(Write{
                    position: Some(Position{
                        start: index,
                        end: index,
                    }),
                })
            }
            ',' => {
                instructions.push(Read{
                    position: Some(Position{
                        start: index,
                        end: index,
                    }),
                })
            }
            '[' => {
                stack.push((instructions, index));
                instructions = vec![];
            }
            ']' => {
                if let Some((mut parent_instr, open_index)) = stack.pop(){
                    parent_instr.push(Loop{
                        body: instructions,
                        position: Some(Position{
                            start: open_index,
                            end: index,
                        }),
                    });
                    instructions = parent_instr;
                } else{
                    return Err(ParseError{
                        message: "This ] has no matching [".to_owned(),
                        position: Position{
                            start: index,
                            end: index,
                        },
                    });
                }
            }
            _ => (),
        }
    }
    if !stack.is_empty(){
        let pos = stack.last().unwrap().1;
        return Err(ParseError{
            message: "This [ has now matching ]".to_owned(),
            position: Position{
                start: pos,
                end: pos,
            },
        });
    }
    Ok(instructions)
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
