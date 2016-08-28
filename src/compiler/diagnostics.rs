use std::fmt;
use ansi_term::Color::{Red, Purple};
use ansi_term::Style;
use ansi_term::ANSIStrings;
use bfir::Position;


#[derive(Debug, PartialEq, Eq)]
pub struct Warning{
    pub message: String,
    pub position: Option<Position>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum Level{
    Warning,
    Error,
}

#[derive(Debug)]
pub struct Info{
    pub level: Level,
    pub filename: String,
    pub message: String,
    pub position: Option<Position>,
    pub source: Option<String>,
}

fn position(s: &str, i: usize) -> (usize, usize){
    let mut char_count = 0;
    for (line_idx, line) in s.split('\n').enumerate(){
        let line_length = line.len();
        if char_count + line_length >= i{
            return (line_idx, i - char_count);
        }
        char_count += line_length + 1;
    }
    unreachable!()
}
