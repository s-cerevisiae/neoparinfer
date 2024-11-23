use std::{collections::HashMap, mem};

use unicode_width::UnicodeWidthStr;

mod legacy;
pub mod parens_only;

pub type ParenTree<'src> = Vec<ParenNode<'src>>;

#[derive(Debug)]
pub struct ParenNode<'src> {
    /// the starting column of the node.
    /// indentation is important.
    col: usize,
    /// when a node appears on the next row,
    /// its position needs to be clamped.
    /// and later operations must be done with this clamped indentation
    row: usize,
    kind: ParenNodeKind<'src>,
}

#[derive(Debug)]
enum ParenNodeKind<'src> {
    Paren(Vec<ParenNode<'src>>),
    Atom(&'src str),
}

// (a b c)
// (ad (a b)\n
//     c)
// keep parens and new lines
// retain the ability to indent and dedent lines
// (a (a (a (b)
//        ccc)))
// (clamp it between two left parens, on the *last* line)
//
// actually paren and indent mode share the same tree?

#[derive(Default)]
struct ReaderState<'src> {
    stash: Vec<Vec<ParenNode<'src>>>,
    cur_node: Vec<ParenNode<'src>>,
    starting_points: Vec<(usize, usize)>,
    in_atom: bool,
}

impl<'src> ReaderState<'src> {
    fn cut_atom(&mut self, line: &'src str, idx: Option<usize>) {
        if self.in_atom {
            let (start, row) = self.starting_points.pop().unwrap();
            let atom = if let Some(end) = idx {
                &line[start..end]
            } else {
                &line[start..]
            };
            self.cur_node.push(ParenNode {
                row,
                col: line[..start].width(),
                kind: ParenNodeKind::Atom(atom),
            });
            self.in_atom = false;
        }
    }

    pub fn read(mut self, input: &'src str) -> ParenTree {
        for (row, line) in input.lines().enumerate() {
            for (idx, c) in line.char_indices() {
                match c {
                    '(' => {
                        self.cut_atom(line, Some(idx));
                        self.starting_points.push((idx, row));
                        self.stash.push(mem::take(&mut self.cur_node));
                    }
                    ')' => {
                        self.cut_atom(line, Some(idx));
                        let partial = self.stash.pop().unwrap();
                        let inner = mem::replace(&mut self.cur_node, partial);
                        let (lidx, row) = self.starting_points.pop().unwrap();
                        self.cur_node.push(ParenNode {
                            col: line[..lidx].width(),
                            row,
                            kind: ParenNodeKind::Paren(inner),
                        });
                    }
                    _ => {
                        if !c.is_whitespace() && !self.in_atom {
                            self.in_atom = true;
                            self.starting_points.push((idx, row));
                        }
                    }
                }
            }

            self.cut_atom(line, None);
        }

        self.cur_node
    }
}

type Operations = HashMap<usize, Indent>;

struct Indent {
    amount: isize,
}

// (a b (c d) e
//  fffff
//  ggggg)
// if col not in between parent start and first parensized sibling start, clamp it and insert the difference
// to operations
fn paren_run(tree: ParenTree<'_>, parent_start: usize, parent_row: usize, sibling_start: Option<usize>) -> Operations {
    for ParenNode { col, row, kind } in tree {
        match kind {
            ParenNodeKind::Paren(_) => todo!(),
            ParenNodeKind::Atom(_) => todo!(),
        }
    }
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader() {
        let reader = ReaderState::default();
        dbg!(reader.read(
            "
(a b
 (c d))"
        ));
    }
}
