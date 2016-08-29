#![warn(trivial_numeric_casts)]
use std::num::Wrapping;
use compiler::bfir::{AstNode, Cell};
use compiler::bfir::AstNode::*;
use compiler::diagnostics::Warning;
use compiler::bounds::highest_cell_index;

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
            start_instr: None,
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

pub fn execute(instrs: &[AstNode], steps: u64) -> (ExecutionState, Option<Warning>){
    let mut state = ExecutionState::initial(instrs);
    let outcome = execute_with_state(instrs, &mut state, steps, None);
    match state.start_instr{
        Some(_) => debug_assert!(!matches!(outcome, Outcome::Completed(_))),
        None => debug_assert!(matches!(outcome, Outcome::Completed(_))),
    }
    match outcome{
        Outcome::RuntimeError(warning) => (state, Some(warning)),
        _ => (state, None)
    }
}

pub fn execute_with_state<'a>(instrs: &'a [AstNode],
                              state: &mut ExecutionState<'a>,
                              steps: u64,
                              dummy_read_value: Option<i8>) -> Outcome{
    let mut steps_left = steps;
    let mut state = state;
    let mut instr_idx = 0;
    while instr_idx < instrs.len() && steps_left > 0{
        let cell_ptr = state.cell_ptr as usize;
        match instrs[instr_idx]{
            Increment{amount, offset, ..} => {
                let target_cell_ptr = (cell_ptr as isize + offset) as usize;
                state.cells[target_cell_ptr] = state.cells[target_cell_ptr] + amount;
                instr_idx += 1;
            }

            Set{amount, offset, ..} => {
                let target_cell_ptr = (cell_ptr as isize + offset) as usize;
                state.cells[target_cell_ptr] = amount;
                instr_idx += 1;
            }

            PointerIncrement{amount, position, ..} => {
                let new_cell_ptr = state.cell_ptr + amount;
                if new_cell_ptr < 0 || new_cell_ptr >= state.cells.len() as isize{
                    state.start_instr = Some(&instrs[instr_idx]);
                    let message = if new_cell_ptr < 0{
                        format!("This instruction moves the pointer to cell {}.",
                        new_cell_ptr).to_owned()
                    } else{
                        format!("This instruction moves the pointer after the last cell ({}), to cell{}.",
                                state.cells.len() - 1,
                                new_cell_ptr)
                            .to_owned()
                    };
                    return Outcome::RuntimeError(Warning{
                        message: message,
                        position: position,
                    });
                } else{
                    state.cell_ptr = new_cell_ptr;
                    instr_idx += 1;
                }
            }
            Write{..} => {
                let cell_value = state.cells[state.cell_ptr as usize];
                state.outputs.push(cell_value.0);
                instr_idx += 1;
            }
            Read{..} => {
                if let Some(read_value) = dummy_read_value{
                    state.cells[state.cell_ptr as usize] = Wrapping(read_value);
                    instr_idx += 1;
                } else {
                    state.start_instr = Some(&instrs[instr_idx]);
                    return Outcome::ReachedRuntimeValue;
                }
            }
            Loop{ref body, ..} => {
                if state.cells[state.cell_ptr as usize].0 == 0{
                    instr_idx += 1;
                } else{
                    let loop_outcome = execute_with_state(body,
                                                          state,
                                                          steps_left,
                                                          dummy_read_value);
                    match loop_outcome{
                        Outcome::Completed(remaining_steps) => {
                            steps_left = remaining_steps;
                        }
                        Outcome::ReachedRuntimeValue |
                        Outcome::RuntimeError(..) |
                        Outcome::OutOfSteps => {
                            if state.start_instr == None{
                                state.start_instr = Some(&instrs[instr_idx]);
                            }
                            return loop_outcome;
                        }
                    }
                }
            }
        }
        MultiplyMove { ref changes, position, .. } => {
            let cell_value = state.cells[cell_ptr];
            if cell_value.0 != 0 {
                for (cell_offset, factor) in changes {
                    let dest_ptr = cell_ptr as isize + *cell_offset;
                    if dest_ptr < 0 {
                        state.start_instr = Some(&instrs[instr_idx]);
                        let message = format!("This multiply loop tried to access cell {} \
                                               (offset {} from current cell {})",
                                              dest_ptr,
                                              *cell_offset,
                                              cell_ptr);
                        return Outcome::RuntimeError(Warning {
                            message: message.to_owned(),
                            position: position,
                        });
                    }
                    if dest_ptr as usize >= state.cells.len() {
                        state.start_instr = Some(&instrs[instr_idx]);
                        return Outcome::RuntimeError(Warning {
                            message: format!("This multiply loop tried to access cell {} (the \
                                              highest cell is {})",
                                             dest_ptr,
                                             state.cells.len() - 1)
                                .to_owned(),
                            position: position,
                        });
                    }
                    let current_val = state.cells[dest_ptr as usize];
                    state.cells[dest_ptr as usize] = current_val + cell_value * (*factor);
                }
                state.cells[cell_ptr] = Wrapping(0);
            }
            instr_idx += 1;
        }
        steps_left -= 1;
    }
    if steps_left == 0{
        if instr_idx < instrs.len(){
            state.start_instr = Some(&instrs[instr_idx]);
        }
            Outcome::OutOfSteps
    } else{
        Outcome::Completed(steps_left)
    }

}
