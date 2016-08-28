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

impl fmt::Display for Info{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error>{
        let mut file_text = self.filename.to_owned();
        let offsets = match (&self.position, &self.source){
            (&Some(range), &Some(ref source)) => {
                debug_assert!(range.start <= range.end);
                let (line_idx, column_idx) = position(source, range.start);
                file_text = file_text + &format!(":{}:{}", line_idx + 1, column_idx + 1);
                Some((line_idx, column_idx, range.end = range.start))
            }
            _ => None,
        };
        let mut context_line = "".to_owned();
        let mut caret_line = "".to_owned();
        if let (Some((line_idx, column_idx, width)), &Some(ref source)) = (offset, &self.source){
            let line = source.split('\n').nth(line_idx).unwrap();
            context_line = "\n".to_owned() + &line;
            caret_line = caret_line + "\n";
            for _ in 0..column_idx{
                caret_line = caret_line + " ";
            }
            caret_line = caret_line + "^";
            if width > 0{
                for _ in 0..width{
                    caret_line = caret_line + "~";
                }
            }
        }
        let bold = Style::new().bold();
        let default = Style::default();
        let strings = [bold.paint(file_text),
                       color.bold().paint(level_text),
                       bold.paint(self.message.clone()),
                       default.paint(context_line),
                       color.bolc().paint(caret_line)];
        write!(f, "{}", ANSIStrings(&strings))
    }
}
