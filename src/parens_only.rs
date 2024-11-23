use std::collections::HashMap;

pub enum Error {
    Unbalanced,
}

type Column = usize;
type Delta = isize;
type Level = usize;

pub struct Line {
    parens: Vec<Paren>,
    indent: Column,
}

#[derive(Clone, Copy)]
pub struct Paren {
    col: Column,
    kind: char,
    side: Side,
}

#[derive(Clone, Copy)]
pub enum Side {
    Opening,
    Closing,
}

pub fn paren_run(input: &[Line]) -> Result<HashMap<usize, Delta>, Error> {
    let mut unclosed_parens: Vec<Paren> = Vec::new();
    let mut first_siblings: HashMap<Level, Paren> = HashMap::new();
    let mut changes = HashMap::new();
    let mut scope_delta = HashMap::new();
    for (row, line) in input.iter().enumerate() {
        let level = unclosed_parens.len();
        let current_delta = scope_delta.get(&level).copied().unwrap_or(0);
        let indent = (line.indent as Delta + current_delta) as Column;
        // calculate deltas
        let min = unclosed_parens.last().map_or(0, |p| p.col);
        let max = first_siblings
            .get(&unclosed_parens.len())
            .map_or(Column::MAX, |p| p.col);
        let delta = indent.clamp(min, max) as Delta - line.indent as Delta;
        if delta != 0 {
            changes.insert(row, delta);
            scope_delta.insert(level, delta);
        }

        // maintain stacks
        for par in &line.parens {
            match par.side {
                Side::Opening => {
                    first_siblings.entry(level).or_insert(*par);
                    unclosed_parens.push(*par);
                }
                Side::Closing => {
                    first_siblings.remove(&level);
                    let opening = unclosed_parens.pop().ok_or(Error::Unbalanced)?;
                    if opening.kind != par.kind {
                        return Err(Error::Unbalanced);
                    }
                }
            }
        }
    }
    Ok(changes)
}
