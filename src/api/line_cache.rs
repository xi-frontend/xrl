// Copyright (C) 2017-2018 Brian Vincent <brainn@gmail.com>
// Copyright (C) 2019-2020 Rasmus Thomsen <oss@cogitri.dev>
// SPDX-License-Identifier: MIT

use crate::protocol::{Annotation, Line, OperationType, Update};
use std::cmp::min;

#[derive(Debug, Default)]
pub struct LineCache {
    pub annotations: Vec<Annotation>,
    pub n_before: u64,
    pub lines: Vec<Option<Line>>,
    pub n_after: u64,
}

impl LineCache {
    pub fn new() -> Self {
        Self {
            annotations: vec![],
            n_before: 0,
            lines: Vec::new(),
            n_after: 0,
        }
    }
    pub fn annotations(&self) -> &Vec<Annotation> {
        &self.annotations
    }
    pub fn height(&self) -> u64 {
        self.n_before + self.lines.len() as u64 + self.n_after
    }
    pub fn width(&self) -> usize {
        self.lines
            .iter()
            .map(|l| match *l {
                None => 0,
                Some(ref l) => l.text.len(),
            })
            .max()
            .unwrap_or(0)
    }
    pub fn get_line(&self, n: u64) -> Option<&Line> {
        if n < self.n_before || n > self.n_before + self.lines.len() as u64 {
            return None;
        }
        let ix = (n - self.n_before) as usize;
        if let Some(&Some(ref line)) = self.lines.get(ix) {
            Some(line)
        } else {
            None
        }
    }
    pub fn get_missing(&self, first: u64, last: u64) -> Vec<(u64, u64)> {
        let mut ret = Vec::new();
        let last = min(last, self.height());
        assert!(first < last);

        let mut run = None;
        for ix in first..last {
            if ix < self.n_before
                || ix >= self.n_before + self.lines.len() as u64
                || self.lines[(ix - self.n_before) as usize].is_none()
            {
                match run {
                    None => {
                        run = Some((ix, ix + 1));
                    }
                    Some((f, l)) if l == ix => {
                        run = Some((f, ix + 1));
                    }
                    Some((f, l)) => {
                        ret.push((f, l));
                        run = Some((ix, ix + 1));
                    }
                }
            }
        }
        if let Some((f, l)) = run {
            ret.push((f, l));
        }
        ret
    }
    /// Handle an xi-core update.
    pub fn update(&mut self, update: Update) {
        let mut new_invalid_before = 0;
        let mut new_lines: Vec<Option<Line>> = Vec::new();
        let mut new_invalid_after = 0;

        let mut old_ix = 0_u64;

        for op in update.operations {
            //debug!("lc before {}-- {} {:?} {}", op_type, new_invalid_before, new_lines, new_invalid_after);
            let n = op.nb_lines;
            match op.operation_type {
                OperationType::Invalidate => {
                    //trace!("invalidate n={}", n);
                    if new_lines.is_empty() {
                        new_invalid_before += n;
                    } else {
                        new_invalid_after += n;
                    }
                }
                OperationType::Insert => {
                    for _ in 0..new_invalid_after {
                        new_lines.push(None)
                    }
                    //trace!("ins n={}", n);
                    new_invalid_after = 0;
                    for line in op.lines {
                        new_lines.push(Some(line));
                    }
                }
                OperationType::Copy => {
                    //trace!("copy n={}", n);

                    for _ in 0..new_invalid_after {
                        new_lines.push(None)
                    }

                    new_invalid_after = 0;

                    let mut n_remaining = n;
                    if old_ix < self.n_before {
                        let invalid = min(n, self.n_before - old_ix);
                        if new_lines.is_empty() {
                            new_invalid_before += invalid;
                        } else {
                            new_invalid_after += invalid;
                        }
                        old_ix += invalid;
                        n_remaining -= invalid;
                    }
                    if n_remaining > 0 && old_ix < self.n_before + self.lines.len() as u64 {
                        let n_copy = min(
                            n_remaining,
                            self.n_before + self.lines.len() as u64 - old_ix,
                        );
                        let start_ix = old_ix - self.n_before;

                        for i in start_ix as usize..(start_ix + n_copy) as usize {
                            if self.lines[i].is_none() {
                                //error!(
                                //    "line {}+{}={}, a copy source is none",
                                //    self.n_invalid_before,
                                //    i,
                                //    self.n_invalid_before + i as u64
                                //);
                            }
                        }
                        new_lines.extend_from_slice(
                            &self.lines[start_ix as usize..(start_ix + n_copy) as usize],
                        );

                        old_ix += n_copy;
                        n_remaining -= n_copy;
                    }
                    if new_lines.is_empty() {
                        new_invalid_before += n_remaining;
                    } else {
                        new_invalid_after += n_remaining;
                    }
                    old_ix += n_remaining;
                }
                OperationType::Skip => {
                    //trace!("skip n={}", n);
                    old_ix += n;
                }
                _ => {}
            }
        }
        self.n_before = new_invalid_before;
        self.lines = new_lines;
        self.n_after = new_invalid_after;
        self.annotations = update.annotations;
        //debug!("lc after update {:?}", self);
    }

    /// Returns true if this Linecache only contains one line, which doesn't contain any text
    pub fn is_empty(&self) -> bool {
        if self.height() == 1 {
            if let Some(line) = self.get_line(0) {
                if &line.text == "" {
                    return true;
                }
            }
        }

        false
    }
}
