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

struct Context {
    opening: Paren,
    first_sibling: Option<Paren>,
    delta: Delta,
}

pub fn paren_run(input: &[Line]) -> Result<HashMap<usize, Delta>, Error> {
    let mut changes = HashMap::new();

    let mut context: Vec<Context> = Vec::new();
    for (row, line) in input.iter().enumerate() {
        let cur_delta = context.last().map_or(0, |l| l.delta);
        let orig_indent = line.indent;
        let cur_indent = (orig_indent as Delta + cur_delta) as Column;
        // calculate delta
        let min = context.last().map_or(0, |l| l.opening.col);
        let max = context
            .last()
            .and_then(|l| l.first_sibling)
            .map_or(Column::MAX, |p| p.col);
        let delta = cur_indent.clamp(min, max) as Delta - orig_indent as Delta;
        if delta != 0 {
            changes.insert(row, delta);
        }

        // maintain stacks
        for par in &line.parens {
            match par.side {
                Side::Opening => {
                    context
                        .last_mut()
                        .map(|l| l.first_sibling.get_or_insert(*par));
                    context.push(Context {
                        opening: *par,
                        first_sibling: None,
                        delta,
                    });
                }
                Side::Closing => {
                    let cur_layer = context.pop().ok_or(Error::Unbalanced)?;
                    // todo: pairing usually doesn't work like that
                    if cur_layer.opening.kind != par.kind {
                        return Err(Error::Unbalanced);
                    }
                }
            }
        }
    }

    Ok(changes)
}
