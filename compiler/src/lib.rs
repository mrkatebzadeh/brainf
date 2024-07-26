use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    Inc(i32),
    Move(i32),
    Out,
    In,
    Clear,
    AddAt(i32, i32),
    MulAdd(Vec<(i32, i32)>),
    Scan(i32),
    Loop(Vec<Op>),
}

pub fn parse_to_ir(src: &str) -> Vec<Op> {
    let mut ops = Vec::new();
    let mut it = src.chars().filter(|c| "+-<>[].,".contains(*c)).peekable();
    while let Some(c) = it.peek().cloned() {
        match c {
            '+' | '-' => {
                let mut n = 0;
                while let Some(&ch) = it.peek() {
                    if ch == '+' {
                        n += 1;
                        it.next();
                    } else if ch == '-' {
                        n -= 1;
                        it.next();
                    } else {
                        break;
                    }
                }
                if n != 0 {
                    ops.push(Op::Inc(n));
                }
            }
            '>' | '<' => {
                let mut n = 0;
                while let Some(&ch) = it.peek() {
                    if ch == '>' {
                        n += 1;
                        it.next();
                    } else if ch == '<' {
                        n -= 1;
                        it.next();
                    } else {
                        break;
                    }
                }
                if n != 0 {
                    ops.push(Op::Move(n));
                }
            }
            '.' => {
                ops.push(Op::Out);
                it.next();
            }
            ',' => {
                ops.push(Op::In);
                it.next();
            }
            '[' => {
                it.next();
                let mut depth = 1;
                let mut body = String::new();
                while let Some(ch) = it.next() {
                    if ch == '[' {
                        depth += 1;
                    } else if ch == ']' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    body.push(ch);
                }
                let inner = parse_to_ir(&body);
                ops.push(Op::Loop(inner));
            }
            ']' => {
                it.next();
            }
            _ => {
                it.next();
            }
        }
    }
    ops
}

pub fn optimize(mut ops: Vec<Op>) -> Vec<Op> {
    // Recurse first
    for op in ops.iter_mut() {
        if let Op::Loop(inner) = op {
            let o = optimize(inner.clone());
            *inner = o;
        }
    }

    // Clear loop: [-] or [+]
    if ops.len() == 1 {
        if let Op::Loop(inner) = &ops[0] {
            if inner.len() == 1 {
                match inner[0] {
                    Op::Inc(n) if n == 1 || n == -1 => return vec![Op::Clear],
                    _ => {}
                }
            }
        }
    }

    // Peephole passes until stable
    let mut changed = true;
    while changed {
        changed = false;
        let mut out = Vec::new();
        let mut i = 0;
        while i < ops.len() {
            match ops[i].clone() {
                Op::Inc(mut n) => {
                    let mut j = i + 1;
                    while j < ops.len() {
                        match ops[j] {
                            Op::Inc(m) => {
                                n += m;
                                j += 1;
                            }
                            _ => break,
                        }
                    }
                    if n != 0 {
                        out.push(Op::Inc(n));
                    }
                    i = j;
                    if j - i > 1 {
                        changed = true;
                    }
                    continue;
                }
                Op::Move(mut n) => {
                    let mut j = i + 1;
                    while j < ops.len() {
                        match ops[j] {
                            Op::Move(m) => {
                                n += m;
                                j += 1;
                            }
                            _ => break,
                        }
                    }
                    if n != 0 {
                        out.push(Op::Move(n));
                    }
                    i = j;
                    if j - i > 1 {
                        changed = true;
                    }
                    continue;
                }
                Op::Loop(inner) => {
                    // Move-add loops like [->+<], [->>+<<], [->+>+<<]
                    if is_move_add_loop(&inner) {
                        let moves = decode_move_add(&inner);
                        out.push(Op::MulAdd(moves));
                        out.push(Op::Clear);
                        changed = true;
                        i += 1;
                        continue;
                    }

                    // Scan loops: [>], [<]
                    if inner.len() == 1 {
                        match inner[0] {
                            Op::Move(d) if d == 1 || d == -1 => {
                                out.push(Op::Scan(d));
                                changed = true;
                                i += 1;
                                continue;
                            }
                            _ => {}
                        }
                    }
                    out.push(Op::Loop(inner));
                    i += 1;
                }
                other => {
                    out.push(other);
                    i += 1;
                }
            }
        }
        ops = out;
    }

    ops
}

fn is_move_add_loop(inner: &[Op]) -> bool {
    // pattern: sequence of Inc/Move ending with Move back to origin and net Inc at origin -1
    // minimal: Inc(-1) then balanced moves and positive adds elsewhere
    let mut pos = 0;
    let mut deltas: Vec<(i32, i32)> = Vec::new();
    for op in inner.iter() {
        match *op {
            Op::Inc(n) => {
                deltas.push((pos, n));
            }
            Op::Move(d) => pos += d,
            _ => return false,
        }
    }
    if pos != 0 {
        return false;
    }
    let mut at0 = 0;
    let mut others = 0;
    for (p, n) in deltas.into_iter() {
        if p == 0 {
            at0 += n;
        } else {
            if n != 0 {
                others += 1;
            }
        }
    }
    at0 == -1 && others > 0
}

fn decode_move_add(inner: &[Op]) -> Vec<(i32, i32)> {
    let mut pos = 0;
    let mut map: std::collections::BTreeMap<i32, i32> = Default::default();
    for op in inner.iter() {
        match *op {
            Op::Inc(n) => {
                *map.entry(pos).or_insert(0) += n;
            }
            Op::Move(d) => pos += d,
            _ => {}
        }
    }
    map.into_iter().filter(|(p, _)| *p != 0).collect()
}

pub fn gen_c(ops: &[Op]) -> String {
    let mut out = String::new();
    out.push_str("#include <stdio.h>\n#include <stdlib.h>\n\n");
    out.push_str("int main(void){\n");
    out.push_str("unsigned char t[30000]; for(int i=0;i<30000;i++) t[i]=0; int p=0;\n");
    emit_c_ops(ops, &mut out, 1);
    out.push_str("return 0;}\n");
    out
}

fn emit_c_ops(ops: &[Op], out: &mut String, _depth: usize) {
    for op in ops.iter() {
        match *op {
            Op::Inc(n) => {
                if n > 0 {
                    out.push_str(&format!("t[p]+={};\n", n));
                } else if n < 0 {
                    out.push_str(&format!("t[p]-={};\n", -n));
                }
            }
            Op::Move(n) => {
                if n > 0 {
                    out.push_str(&format!("p+={};\n", n));
                } else if n < 0 {
                    out.push_str(&format!("p-={};\n", -n));
                }
            }
            Op::Out => out.push_str("putchar(t[p]);\n"),
            Op::In => out.push_str("{int c=getchar(); if(c!=EOF) t[p]=c; }\n"),
            Op::Clear => out.push_str("t[p]=0;\n"),
            Op::AddAt(off, amt) => {
                if amt != 0 {
                    if off >= 0 {
                        out.push_str(&format!("t[p+{}]+={};\n", off, amt));
                    } else {
                        out.push_str(&format!("t[p-{}]+={};\n", -off, amt));
                    }
                }
            }
            Op::Scan(d) => {
                if d > 0 {
                    out.push_str("while(t[p]) p++;\n");
                } else {
                    out.push_str("while(t[p]) p--;\n");
                }
            }
            Op::MulAdd(ref v) => {
                for &(off, amt) in v.iter() {
                    if off >= 0 {
                        out.push_str(&format!("t[p+{}]+=t[p]*{};\n", off, amt));
                    } else {
                        out.push_str(&format!("t[p-{}]+=t[p]*{};\n", -off, amt));
                    }
                }
            }
            Op::Loop(ref inner) => {
                out.push_str("while(t[p]){\n");
                emit_c_ops(inner, out, _depth + 1);
                out.push_str("}\n");
            }
        }
    }
}

pub fn compile_to_exe(src: &str, out_exe: &Path) -> Result<()> {
    let ir = parse_to_ir(src);
    let ir = optimize(ir);
    let code = gen_c(&ir);
    let mut c_path = PathBuf::from(out_exe);
    c_path.set_extension("c");
    fs::write(&c_path, code)?;
    let status = Command::new("cc")
        .arg(&c_path)
        .arg("-O2")
        .arg("-o")
        .arg(out_exe)
        .status()?;
    if !status.success() {
        anyhow::bail!("cc failed");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clear_loop() {
        let ir = parse_to_ir("[-]");
        let opt = optimize(ir);
        assert_eq!(opt, vec![Op::Clear]);
    }

    #[test]
    fn move_add_basic() {
        let ir = parse_to_ir("[->+<]");
        let opt = optimize(ir);
        assert_eq!(opt, vec![Op::MulAdd(vec![(1, 1)]), Op::Clear]);
    }

    #[test]
    fn scan_right() {
        let ir = parse_to_ir("[>]");
        let opt = optimize(ir);
        assert_eq!(opt, vec![Op::Scan(1)]);
    }

    #[test]
    fn e2e_hello() -> Result<()> {
        let src = "++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.>+.+++++++..+++.>++.<<+++++++++++++++.>.+++.------.--------.>+.>.";
        let dir = tempfile::tempdir()?;
        let exe = dir.path().join("hello");
        compile_to_exe(src, &exe)?;
        let out = Command::new(&exe).output()?;
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout), "Hello World!\n");
        Ok(())
    }
}
