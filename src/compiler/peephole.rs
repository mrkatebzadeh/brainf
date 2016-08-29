
use std::hash::Hash;
use std::collections::{HashMap, HashSet};
use std::num::Wrapping;

use itertools::Itertools;

use compiler::diagnostics::Warning;

use compiler::bfir::{AstNode, Position, Combine, Cell, get_position};
use compiler::bfir::AstNode::*;

const MAX_OPT_ITERATIONS: u64 = 40;

pub fn optimize(instrs: Vec<AstNode>,
                pass_specification: &Option<String>)
                -> (Vec<AstNode>, Vec<Warning>) {
    let mut prev = instrs.clone();
    let mut warnings = vec![];

    let (mut result, warning) = optimize_once(instrs, pass_specification);

    if let Some(warning) = warning {
        warnings.push(warning);
    }

    for _ in 0..MAX_OPT_ITERATIONS {
        if prev == result {
            return (result, warnings);
        } else {
            prev = result.clone();

            let (new_result, new_warning) = optimize_once(result, pass_specification);

            if let Some(warning) = new_warning {
                warnings.push(warning);
            }
            result = new_result;
        }
    }

    println!("Warning: ran peephole optimisations {} times but did not reach a fixed point!",
             MAX_OPT_ITERATIONS);

    (result, warnings)
}

fn optimize_once(instrs: Vec<AstNode>,
                 pass_specification: &Option<String>)
                 -> (Vec<AstNode>, Option<Warning>) {
    let pass_specification = pass_specification.clone()
                                               .unwrap_or("combine_inc,combine_ptr,known_zero,\
                                                           multiply,zeroing_loop,combine_set,\
                                                           dead_loop,redundant_set,read_clobber,\
                                                           pure_removal,offset_sort"
                                                              .to_owned());
    let passes: Vec<_> = pass_specification.split(",").collect();

    let mut instrs = instrs;

    if passes.contains(&"combine_inc") {
        instrs = combine_increments(instrs);
    }
    if passes.contains(&"combine_ptr") {
        instrs = combine_ptr_increments(instrs);
    }
    if passes.contains(&"known_zero") {
        instrs = annotate_known_zero(instrs);
    }
    if passes.contains(&"multiply") {
        instrs = extract_multiply(instrs);
    }
    if passes.contains(&"zeroing_loop") {
        instrs = zeroing_loops(instrs);
    }
    if passes.contains(&"combine_set") {
        instrs = combine_set_and_increments(instrs);
    }
    if passes.contains(&"dead_loop") {
        instrs = remove_dead_loops(instrs);
    }
    if passes.contains(&"redundant_set") {
        instrs = remove_redundant_sets(instrs);
    }
    if passes.contains(&"read_clobber") {
        instrs = remove_read_clobber(instrs);
    }
    let warning;
    if passes.contains(&"pure_removal") {
        let (removed, pure_warning) = remove_pure_code(instrs);
        instrs = removed;
        warning = pure_warning;
    } else {
        warning = None;
    }

    if passes.contains(&"offset_sort") {
        instrs = sort_by_offset(instrs);
    }

    (instrs, warning)
}

trait MapLoopsExt: Iterator<Item=AstNode> {
    fn map_loops<F>(&mut self, f: F) -> Vec<AstNode>
        where F: Fn(Vec<AstNode>) -> Vec<AstNode>
    {
        self.map(|instr| {
                match instr {
                    Loop { body, position } => {
                        Loop {
                            body: f(body),
                            position: position,
                        }
                    }
                    other => other,
                }
            })
            .collect()
    }
}

impl<I> MapLoopsExt for I where I: Iterator<Item = AstNode>
{}

pub fn previous_cell_change(instrs: &[AstNode], index: usize) -> Option<usize> {
    assert!(index < instrs.len());

    let mut needed_offset = 0;
    for i in (0..index).rev() {
        match instrs[i] {
            Increment { offset, .. } => {
                if offset == needed_offset {
                    return Some(i);
                }
            }
            Set { offset, .. } => {
                if offset == needed_offset {
                    return Some(i);
                }
            }
            PointerIncrement { amount, .. } => {
                needed_offset += amount;
            }
            MultiplyMove { ref changes, .. } => {
                let mut offsets: Vec<isize> = changes.keys()
                                                     .into_iter()
                                                     .cloned()
                                                     .collect();
                offsets.push(0);

                if offsets.contains(&needed_offset) {
                    return Some(i);
                }
            }
            Write {..} => {}
            Read {..} | Loop {..} => return None,
        }
    }
    None
}

pub fn next_cell_change(instrs: &[AstNode], index: usize) -> Option<usize> {
    
    assert!(index < instrs.len());

    let mut needed_offset = 0;
    for (i, instr) in instrs.iter().enumerate().skip(index + 1) {
        match *instr {
            Increment { offset, .. } => {
                if offset == needed_offset {
                    return Some(i);
                }
            }
            Set { offset, .. } => {
                if offset == needed_offset {
                    return Some(i);
                }
            }
            PointerIncrement { amount, .. } => {
                needed_offset -= amount;
            }
            MultiplyMove { ref changes, .. } => {
                let mut offsets: Vec<isize> = changes.keys()
                                                     .into_iter()
                                                     .cloned()
                                                     .collect();
                offsets.push(0);

                if offsets.contains(&needed_offset) {
                    return Some(i);
                }
            }
            Write {..} => {}
            Read {..} | Loop {..} => return None,
        }
    }
    None
}

pub fn combine_increments(instrs: Vec<AstNode>) -> Vec<AstNode> {
    instrs.into_iter()
          .coalesce(|prev_instr, instr| {
              if let Increment { amount: prev_amount, offset: prev_offset, position: prev_pos } =
                     prev_instr {
                  if let Increment { amount, offset, position } = instr {
                      if prev_offset == offset {
                          return Ok(Increment {
                              amount: amount + prev_amount,
                              offset: offset,
                              position: prev_pos.combine(position),
                          });
                      }
                  }
              }
              Err((prev_instr, instr))
          })
          .filter(|instr| {
              if let Increment { amount: Wrapping(0), .. } = *instr {
                  return false;
              }
              true
          })
          .map_loops(combine_increments)
}

pub fn combine_ptr_increments(instrs: Vec<AstNode>) -> Vec<AstNode> {
    instrs.into_iter()
          .coalesce(|prev_instr, instr| {
              if let PointerIncrement { amount: prev_amount, position: prev_pos } = prev_instr {
                  if let PointerIncrement { amount, position } = instr {
                      return Ok(PointerIncrement {
                          amount: amount + prev_amount,
                          position: prev_pos.combine(position),
                      });
                  }
              }
              Err((prev_instr, instr))
          })
          .filter(|instr| {
              if let PointerIncrement { amount: 0, .. } = *instr {
                  return false;
              }
              true
          })
          .map_loops(combine_ptr_increments)
}

pub fn remove_read_clobber(instrs: Vec<AstNode>) -> Vec<AstNode> {
    let mut redundant_instr_positions = HashSet::new();
    let mut last_write_index = None;

    for (index, instr) in instrs.iter().enumerate() {
        match *instr {
            Read {..} => {
                if let Some(prev_modify_index) = previous_cell_change(&instrs, index) {
                    if let Some(write_index) = last_write_index {
                        if write_index > prev_modify_index {
                            continue;
                        }
                    }
                    if matches!(instrs[prev_modify_index], MultiplyMove { ..}) {
                        continue;
                    }

                    redundant_instr_positions.insert(prev_modify_index);
                }
            }
            Write {..} => {
                last_write_index = Some(index);
            }
            _ => {}
        }
    }

    instrs.into_iter()
          .enumerate()
          .filter(|&(index, _)| !redundant_instr_positions.contains(&index))
          .map(|(_, instr)| instr)
          .map_loops(remove_read_clobber)
}

pub fn zeroing_loops(instrs: Vec<AstNode>) -> Vec<AstNode> {
    instrs.into_iter()
          .map(|instr| {
              if let Loop { ref body, position } = instr {
                  // If the loop is [-]
                  if body.len() == 1 {
                      if let Increment { amount: Wrapping(-1), offset: 0, .. } = body[0] {
                          return Set {
                              amount: Wrapping(0),
                              offset: 0,
                              position: position,
                          };
                      }
                  }
              }
              instr
          })
          .map_loops(zeroing_loops)
}

pub fn remove_dead_loops(instrs: Vec<AstNode>) -> Vec<AstNode> {
    instrs.clone()
          .into_iter()
          .enumerate()
          .filter(|&(index, ref instr)| {
              match *instr {
                  Loop {..} => {}
                  _ => {
                      return true;
                  }
              }

              if let Some(prev_change_index) = previous_cell_change(&instrs, index) {
                  let prev_instr = &instrs[prev_change_index];
                  if let Set { amount: Wrapping(0), offset: 0, .. } = *prev_instr {
                      return false;
                  }
              }
              true
          })
          .map(|(_, instr)| instr)
          .map_loops(remove_dead_loops)
}

pub fn sort_by_offset(instrs: Vec<AstNode>) -> Vec<AstNode> {
    let mut sequence = vec![];
    let mut result = vec![];

    for instr in instrs {
        match instr {
            Increment {..} | Set {..} | PointerIncrement {..} => {
                sequence.push(instr);
            }
            _ => {
                if !sequence.is_empty() {
                    result.extend(sort_sequence_by_offset(sequence));
                    sequence = vec![];
                }
                if let Loop { body, position } = instr {
                    result.push(Loop {
                        body: sort_by_offset(body),
                        position: position,
                    });
                } else {
                    result.push(instr);
                }
            }
        }
    }

    if !sequence.is_empty() {
        result.extend(sort_sequence_by_offset(sequence));
    }

    result
}

fn ordered_values<K: Ord + Hash + Eq, V>(map: HashMap<K, V>) -> Vec<V> {
    let mut items: Vec<_> = map.into_iter().collect();
    items.sort_by(|a, b| a.0.cmp(&b.0));
    items.into_iter().map(|(_, v)| v).collect()
}

fn sort_sequence_by_offset(instrs: Vec<AstNode>) -> Vec<AstNode> {
    let mut instrs_by_offset: HashMap<isize, Vec<AstNode>> = HashMap::new();
    let mut current_offset = 0;
    let mut last_ptr_inc_pos = None;

    for instr in instrs {
        match instr {
            Increment { amount, offset, position } => {
                let new_offset = offset + current_offset;
                let same_offset_instrs = instrs_by_offset.entry(new_offset).or_insert_with(|| vec![]);
                same_offset_instrs.push(Increment {
                    amount: amount,
                    offset: new_offset,
                    position: position,
                });
            }
            Set { amount, offset, position } => {
                let new_offset = offset + current_offset;
                let same_offset_instrs = instrs_by_offset.entry(new_offset).or_insert_with(|| vec![]);
                same_offset_instrs.push(Set {
                    amount: amount,
                    offset: new_offset,
                    position: position,
                });
            }
            PointerIncrement { amount, position } => {
                current_offset += amount;
                last_ptr_inc_pos = Some(position);
            }
            _ => unreachable!(),
        }
    }

    let mut results: Vec<AstNode> = vec![];
    for same_offset_instrs in ordered_values(instrs_by_offset) {
        results.extend(same_offset_instrs.into_iter());
    }

    if current_offset != 0 {
        results.push(PointerIncrement {
            amount: current_offset,
            position: last_ptr_inc_pos.unwrap(),
        });
    }
    results
}

pub fn combine_set_and_increments(instrs: Vec<AstNode>) -> Vec<AstNode> {
    instrs.into_iter()
          .coalesce(|prev_instr, instr| {
              if let (&Increment { offset: inc_offset, position: inc_pos, .. },
                      &Set { amount: set_amount, offset: set_offset, position: set_pos }) =
                     (&prev_instr, &instr) {
                  if inc_offset == set_offset {
                      return Ok(Set {
                          amount: set_amount,
                          offset: set_offset,
                          position: set_pos.combine(inc_pos),
                      });
                  }
              }
              Err((prev_instr, instr))
          })
          .coalesce(|prev_instr, instr| {
              if let Set { amount: set_amount, offset: set_offset, position: set_pos } =
                     prev_instr {
                  if let Increment { amount: inc_amount, offset: inc_offset, position: inc_pos } =
                         instr {
                      if inc_offset == set_offset {
                          return Ok(Set {
                              amount: set_amount + inc_amount,
                              offset: set_offset,
                              position: set_pos.combine(inc_pos),
                          });
                      }
                  }
              }
              Err((prev_instr, instr))
          })
          .coalesce(|prev_instr, instr| {
              if let (&Set { offset: offset1, position: position1, .. },
                      &Set { amount, offset: offset2, position: position2 }) = (&prev_instr,
                                                                                &instr) {
                  if offset1 == offset2 {
                      return Ok(Set {
                          amount: amount,
                          offset: offset1,
                          position: position1.combine(position2),
                      });
                  }
              }
              Err((prev_instr, instr))
          })
          .map_loops(combine_set_and_increments)
}

pub fn remove_redundant_sets(instrs: Vec<AstNode>) -> Vec<AstNode> {
    let mut reduced = remove_redundant_sets_inner(instrs);
    if let Some(&Set { amount: Wrapping(0), offset: 0, .. }) = reduced.first() {
        reduced.remove(0);
    }

    reduced
}

fn remove_redundant_sets_inner(instrs: Vec<AstNode>) -> Vec<AstNode> {
    let mut redundant_instr_positions = HashSet::new();

    for (index, instr) in instrs.iter().enumerate() {
        match *instr {
            Loop {..} | MultiplyMove {..} => {
                if let Some(next_index) = next_cell_change(&instrs, index) {
                    if let Set { amount: Wrapping(0), offset: 0, .. } = instrs[next_index] {
                        redundant_instr_positions.insert(next_index);
                    }
                }
            }
            _ => {}
        }
    }

    instrs.into_iter()
          .enumerate()
          .filter(|&(index, _)| !redundant_instr_positions.contains(&index))
          .map(|(_, instr)| instr)
          .map_loops(remove_redundant_sets_inner)
}

pub fn annotate_known_zero(instrs: Vec<AstNode>) -> Vec<AstNode> {
    let mut result = vec![];

    let position = if instrs.is_empty() {
        None
    } else {
        get_position(&instrs[0]).map(|first_instr_pos| {
            Position {
                start: first_instr_pos.start,
                end: first_instr_pos.start,
            }
        })
    };

    let set_instr = Set {
        amount: Wrapping(0),
        offset: 0,
        position: position,
    };
    if instrs.first() != Some(&set_instr) {
        result.push(set_instr);
    }

    result.extend(annotate_known_zero_inner(instrs));
    result
}

fn annotate_known_zero_inner(instrs: Vec<AstNode>) -> Vec<AstNode> {
    let mut result = vec![];

    for (i, instr) in instrs.iter().enumerate() {
        let instr = instr.clone();

        match instr {
            Loop { body, position } => {
                result.push(Loop {
                    body: annotate_known_zero_inner(body),
                    position: position,
                });
                let set_pos = position.map(|loop_pos| {
                    Position {
                        start: loop_pos.end,
                        end: loop_pos.end,
                    }
                });

                let set_instr = Set {
                    amount: Wrapping(0),
                    offset: 0,
                    position: set_pos,
                };
                if instrs.get(i + 1) != Some(&set_instr) {
                    result.push(set_instr.clone());
                }
            }
            _ => {
                result.push(instr);
            }
        }
    }

    result
}

pub fn remove_pure_code(mut instrs: Vec<AstNode>) -> (Vec<AstNode>, Option<Warning>) {
    let mut pure_instrs = vec![];
    while !instrs.is_empty() {
        let last_instr = instrs.pop().unwrap();

        match last_instr {
            Read {..} | Write {..} | Loop {..} => {
                instrs.push(last_instr);
                break;
            }
            _ => {
                pure_instrs.push(last_instr);
            }
        }
    }

    let warning = if pure_instrs.is_empty() {
        None
    } else {
        let position = pure_instrs.into_iter()
                                  .map(|instr| get_position(&instr))
                                  .filter(|pos| pos.is_some())
                                  .fold1(|pos1, pos2| pos1.combine(pos2))
                                  .map(|pos| pos.unwrap());
        Some(Warning {
            message: "These instructions have no effect.".to_owned(),
            position: position,
        })
    };

    (instrs, warning)
}

fn is_multiply_loop_body(body: &[AstNode]) -> bool {
    for body_instr in body {
        match *body_instr {
            Increment {..} => {}
            PointerIncrement {..} => {}
            _ => return false,
        }
    }

    let mut net_movement = 0;
    for body_instr in body {
        if let PointerIncrement { amount, .. } = *body_instr {
            net_movement += amount;
        }
    }
    if net_movement != 0 {
        return false;
    }

    let changes = cell_changes(body);
    if let Some(&Wrapping(-1)) = changes.get(&0) {
    } else {
        return false;
    }

    changes.len() >= 2
}

fn cell_changes(instrs: &[AstNode]) -> HashMap<isize, Cell> {
    let mut changes = HashMap::new();
    let mut cell_index: isize = 0;

    for instr in instrs {
        match *instr {
            Increment { amount, offset, .. } => {
                let current_amount = *changes.get(&(cell_index + offset)).unwrap_or(&Wrapping(0));
                changes.insert(cell_index, current_amount + amount);
            }
            PointerIncrement { amount, .. } => {
                cell_index += amount;
            }
            _ => unreachable!(),
        }
    }

    changes
}

pub fn extract_multiply(instrs: Vec<AstNode>) -> Vec<AstNode> {
    instrs.into_iter()
          .map(|instr| {
              match instr {
                  Loop { body, position } => {
                      if is_multiply_loop_body(&body) {
                          let mut changes = cell_changes(&body);
                          changes.remove(&0);

                          MultiplyMove {
                              changes: changes,
                              position: position,
                          }
                      } else {
                          Loop {
                              body: extract_multiply(body),
                              position: position,
                          }
                      }
                  }
                  i => i,
              }
          })
          .collect()
}
