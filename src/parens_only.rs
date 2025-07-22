use std::{
    collections::{HashMap, VecDeque},
    iter,
};

#[derive(Debug)]
pub enum Error {
    Unbalanced,
}

type Column = usize;
type Delta = isize;

#[derive(Clone, Debug)]
pub struct Line {
    parens: Vec<Paren>,
    indent: Column,
}

#[derive(Clone, Copy, Debug)]
pub struct Paren {
    col: Column,
    kind: char,
    side: Side,
    mid_line: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum Side {
    Opening,
    Closing,
}

#[derive(Debug)]
struct Context {
    opening: Paren,
    delta: Delta,
}

/// A "parentheses" run to fix indentation based on parens.
/// An implementation of "paren mode" in Parinfer.
pub fn paren_run(input: &[Line]) -> Result<HashMap<usize, Delta>, Error> {
    let mut changes = HashMap::new();

    let mut max_indent = Column::MAX;
    let mut context: Vec<Context> = Vec::new();
    for (row, line) in input.iter().enumerate() {
        let cur_delta = context.last().map_or(0, |l| l.delta);
        let orig_indent = line.indent;
        // propagates previous indentation changes to current line
        let cur_indent = orig_indent.saturating_add_signed(cur_delta);
        // calculate delta
        let min = context.last().map_or(0, |l| l.opening.col + 1);
        let max = max_indent
            // the case where first_sibling is clamped
            .max(min);
        let delta = cur_indent.clamp(min, max) as Delta - orig_indent as Delta;
        if delta != 0 {
            changes.insert(row, delta);
        }
        // this mimics the original parinfer clamping
        // max_indent = Column::MAX;

        // maintain stacks
        for par in &line.parens {
            match par.side {
                Side::Opening => {
                    let par = Paren {
                        col: par.col.saturating_add_signed(delta),
                        ..*par
                    };
                    context.push(Context {
                        opening: par,
                        delta,
                    });
                    max_indent = Column::MAX;
                }
                Side::Closing => {
                    let cur_layer = context.pop().ok_or(Error::Unbalanced)?;
                    // todo: pairing usually doesn't work like that
                    if cur_layer.opening.kind != par.kind {
                        return Err(Error::Unbalanced);
                    }
                    if !par.mid_line {
                        max_indent = cur_layer.opening.col;
                    } else {
                        max_indent = Column::MAX;
                    }
                }
            }
        }
    }

    Ok(changes)
}

pub fn indent_run(input: &[Line]) -> (Vec<(usize, Paren)>, Vec<usize>) {
    // assumption: indentation here is in increasing order. otherwise they are already solved.
    let mut unpaired_lparen: Vec<Paren> = Vec::new();
    let mut paired: VecDeque<(Paren, Paren, usize)> = VecDeque::new();
    let mut to_be_deleted = Vec::new();
    let mut to_be_added = Vec::new();

    for (row, line) in input
        .iter()
        .chain(iter::once(&Line {
            parens: Vec::new(),
            indent: 0,
        }))
        .enumerate()
    {
        // iterate over unpaired left-parens, push a paren to the last line if current line indents
        // further than lparen itself.
        // (stop when there's an corresponding rparen)
        while let Some(lp) = unpaired_lparen.last() {
            if line.indent <= lp.col {
                to_be_added.push(row - 1);
                unpaired_lparen.pop();
            } else {
                break;
            }
        }

        // iterate over paired parens,
        // TODO: reverse it
        while let Some((lp, rp, rp_row)) = paired.front() {
            if line.indent <= lp.col {
                if row - 1 != *rp_row {
                    to_be_added.push(row - 1);
                    to_be_deleted.push((*rp_row, *rp));
                }
                paired.pop_front();
            } else {
                break;
            }
        }

        for p in &line.parens {
            match p.side {
                Side::Opening => {
                    unpaired_lparen.push(*p);
                }
                Side::Closing => {
                    if let Some(lp) = unpaired_lparen.pop()
                        && lp.kind == p.kind
                    {
                        if !p.mid_line {
                            paired.push_back((lp, *p, row));
                        }
                    } else {
                        to_be_deleted.push((row, *p));
                    }
                }
            }
        }
    }

    (to_be_deleted, to_be_added)
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use indoc::indoc;

    use super::*;

    fn simple_parse(input: &str) -> Vec<Line> {
        input
            .lines()
            .map(|input_line| {
                let trimmed = input_line.trim_start_matches(' ');
                let indent = input_line.len() - trimmed.len();
                let mut parens: Vec<Paren> = Vec::new();
                for (col, c) in input_line.chars().enumerate() {
                    let side = match c {
                        '(' => Side::Opening,
                        ')' => Side::Closing,
                        _ => {
                            if let Some(p) = parens.last_mut() {
                                p.mid_line = true;
                            }
                            continue;
                        }
                    };
                    parens.push(Paren {
                        kind: '(',
                        col,
                        side,
                        mid_line: false,
                    })
                }
                Line { parens, indent }
            })
            .collect()
    }

    fn apply_indent_changes(input: &str, changes: &HashMap<usize, Delta>) -> String {
        let mut result = String::new();
        for (i, l) in input.lines().enumerate() {
            let delta = changes.get(&i).copied().unwrap_or(0);
            if delta < 0 {
                result.push_str(&l[delta.unsigned_abs()..]);
            } else {
                result.push_str(&" ".repeat(delta as usize));
                result.push_str(l);
            }
            result.push('\n');
        }
        result.pop();
        result
    }

    fn fix_by_paren(input: &str) -> String {
        let changes = paren_run(&simple_parse(input)).unwrap();
        apply_indent_changes(input, &changes)
    }

    #[test]
    fn test_simple_parse() {
        expect![[r#"
            [
                Line {
                    parens: [
                        Paren {
                            col: 0,
                            kind: '(',
                            side: Opening,
                            mid_line: true,
                        },
                        Paren {
                            col: 2,
                            kind: '(',
                            side: Closing,
                            mid_line: false,
                        },
                    ],
                    indent: 0,
                },
            ]
        "#]]
        .assert_debug_eq(&simple_parse("(a)"));

        expect![[r#"
            [
                Line {
                    parens: [],
                    indent: 1,
                },
            ]
        "#]]
        .assert_debug_eq(&simple_parse(" a"));

        expect![[r#"
            [
                Line {
                    parens: [
                        Paren {
                            col: 0,
                            kind: '(',
                            side: Opening,
                            mid_line: false,
                        },
                    ],
                    indent: 0,
                },
                Line {
                    parens: [
                        Paren {
                            col: 1,
                            kind: '(',
                            side: Closing,
                            mid_line: false,
                        },
                    ],
                    indent: 1,
                },
            ]
        "#]]
        .assert_debug_eq(&simple_parse(
            r"(
 )",
        ));
    }

    #[test]
    fn test_paren_mode() {
        let input = "()";
        expect!["()"].assert_eq(&fix_by_paren(input));

        let input = r"(
)";
        expect![[r#"
            (
             )"#]]
        .assert_eq(&fix_by_paren(input));

        let input = r"
(
a b
 c)"
        .trim_start();
        expect![[r#"
            (
             a b
             c)"#]]
        .assert_eq(&fix_by_paren(input));

        let input = r"
(im a haskell user (who prefer
    (code like this)
))"
        .trim_start();
        expect![[r#"
            (im a haskell user (who prefer
                                (code like this)
                                ))"#]]
        .assert_eq(&fix_by_paren(input));

        let input = r"
(
(
)
)"
        .trim_start();
        expect![[r#"
            (
             (
              )
             )"#]]
        .assert_eq(&fix_by_paren(input));

        let input = r"
(im a c user (
    who prefer
 (
   code like this
 )
))"
        .trim_start();
        expect![[r#"
            (im a c user (
                          who prefer
                          (
                            code like this
                           )
                          ))"#]]
        .assert_eq(&fix_by_paren(input));

        let input = r"
(defn foo
  ((a)
    (foo a 1))
  ((a b)
    (let (sum (+ a b)
          prod (* a b)
          result ( ; gather vals
            :sum sum
            :prod prod
          ))
      result)
    ; TODO: something
    ))"
        .trim_start();
        expect![[r#"
            (defn foo
              ((a)
               (foo a 1))
              ((a b)
               (let (sum (+ a b)
                     prod (* a b)
                     result ( ; gather vals
                             :sum sum
                             :prod prod
                             ))
                 result)
               ; TODO: something
               ))"#]]
        .assert_eq(&fix_by_paren(input));
    }

    #[test]
    fn first_sibling_not_enough() {
        let input = r"
(    (a b)
  (c d)
    e
 )"
        .trim_start();
        expect![[r#"
            (    (a b)
              (c d)
              e
             )"#]]
        .assert_eq(&fix_by_paren(input));

        let input = r"
((a) (b)
     (c d)
     e)"
        .trim_start();
        expect![[r#"
            ((a) (b)
                 (c d)
                 e)"#]]
        .assert_eq(&fix_by_paren(input));
    }

    #[test]
    fn pull_back() {
        let input = r"
()
 b"
        .trim_start();
        expect![[r#"
            ()
            b"#]]
        .assert_eq(&fix_by_paren(input));

        let input = r"
(a (b) c
       d)"
        .trim_start();
        expect![[r#"
            (a (b) c
                   d)"#]]
        .assert_eq(&fix_by_paren(input));

        let input = r"
(a (b)
     c
       d)"
        .trim_start();
        expect![[r#"
            (a (b)
               c
               d)"#]]
        .assert_eq(&fix_by_paren(input));
    }

    fn apply_paren_changes(input: &str, changes: &(Vec<(usize, Paren)>, Vec<usize>)) -> String {
        let mut result = Vec::new();
        let (del, add) = changes;
        for (row, line) in input.lines().enumerate() {
            let mut line = line.to_owned();
            let mut to_remove = Vec::new();
            for (r, p) in del.iter().rev() {
                if *r == row {
                    to_remove.push(p.col);
                }
            }
            to_remove.sort_unstable();
            to_remove.iter().rev().for_each(|i| {
                line.remove(*i);
            });
            let count_append = add.iter().filter(|&&r| r == row).count();
            line.push_str(&")".repeat(count_append));
            result.push(line);
        }
        result.join("\n")
    }

    fn fix_by_indent(input: &str) -> String {
        apply_paren_changes(input, &indent_run(&simple_parse(input)))
    }

    #[test]
    fn test_indent_mode() {
        let input = "()";
        expect!["()"].assert_eq(&fix_by_indent(input));

        let input = "(";
        expect!["()"].assert_eq(&fix_by_indent(input));

        let input = "())";
        expect!["()"].assert_eq(&fix_by_indent(input));

        let input = indoc! {r"
            (well (known fact
              (lisp
                (is)
                  indentation
                  based
        "};
        expect![[r#"
            (well (known fact)
              (lisp
                (is
                  indentation
                  based)))"#]]
        .assert_eq(&fix_by_indent(input));

        let input = indoc! {r"
            (paired
              (paired))
             (move this)
        "};
        expect![[r#"
            (paired
              (paired)
             (move this))"#]]
        .assert_eq(&fix_by_indent(input));
    }
}
