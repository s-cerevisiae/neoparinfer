#![allow(unused)]

pub mod error;

use std::{borrow::Cow, collections::HashMap};

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

#[derive(PartialEq, Eq)]
pub enum Mode {
    Indent,
    Paren,
}
pub struct Options {
    mode: Mode,
    smart: bool,
}

type Row = usize;
type Column = usize;
type Delta = isize;

#[derive(Default)]
pub struct Pos {
    row: Row,
    col: Column,
}

#[derive(Default)]
pub struct LineState {
    indent_x: Column,
    comment_x: Column,
    indent_delta: Delta,
    tracking_indent: bool,
}
// TODO:
// result
//     .error_pos_cache
//     .remove(&ErrorName::UnmatchedCloseParen);
// result
//     .error_pos_cache
//     .remove(&ErrorName::UnmatchedOpenParen);
// result.error_pos_cache.remove(&ErrorName::LeadingCloseParen);

// result.tracking_arg_tab_stop = TrackingArgTabStop::NotSearching;
// result.tracking_indent = !result.is_in_stringish();

pub struct Change {
    old_end_col: Column,
    new_end_col: Column,
    lookup_row: Row,
    lookup_col: Column,
}

type Changes = HashMap<(Row, Column), Change>;

pub struct TabStop<'a> {
    pub ch: &'a str,
    pub col: Column,
    pub row: Row,
    pub arg_x: Option<Column>,
}

#[allow(unused)]
pub fn process_text(input: &str, options: &Options, changes: &Changes) {
    let mut result: Vec<Cow<str>> = Vec::new();
    for (row, line) in input.lines().enumerate() {
        result.push(line.into());
        let line_state = LineState::default();

        todo!("set tabstops");

        for (col, c) in line.graphemes(true).scan(0, |col, c| {
            let cur_col = *col;
            *col += c.width();
            Some((cur_col, c))
        }) {
            let pos = Pos { row, col };
            handle_change_delta(&mut line_state, &pos, options, changes);
            process_char(c, &pos, &mut line_state, options);
        }
    }
}

fn handle_change_delta(
    line_state: &mut LineState,
    pos: &Pos,
    options: &Options,
    changes: &HashMap<(usize, usize), Change>,
) {
    if (options.smart || options.mode == Mode::Paren)
        && let Some(change) = changes.get(&(pos.row, pos.col))
    {
        line_state.indent_delta += change.new_end_col as Delta - change.old_end_col as Delta;
    }
}

fn process_char(
    c: &str,
    pos: &Pos,
    line_state: &mut LineState,
    options: &Options,
) -> Result<(), error::Error> {
    todo!()
}
