use crate::{Line, Operation, OperationType, Update};
use std::collections::HashMap;

/// Line cache struct to work with xi update protocol.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LineCache {
    invalid_before: u64,
    lines: HashMap<u64, Line>,
    invalid_after: u64,
}

impl LineCache {
    /// Retrieve the number of invalid lines before
    /// the start of the line cache.
    pub fn before(&self) -> u64 {
        self.invalid_before
    }

    /// Retrieve the number of invalid lines after
    /// the line cache.
    pub fn after(&self) -> u64 {
        self.invalid_after
    }

    /// Retrieve all lines in the cache.
    pub fn lines(&self) -> &HashMap<u64, Line> {
        &self.lines
    }

    /// Retrieve the total height of the linecache
    pub fn height(&self) -> u64 {
        self.before() + self.lines.len() as u64 + self.after()
    }

    /// Handle an xi-core update.
    pub fn update(&mut self, update: Update) {
        debug!("line cache before update: {:?}", self);
        debug!(
            "operations to be applied to the line cache: {:?}",
            &update.operations
        );
        let LineCache {
            ref mut lines,
            ref mut invalid_before,
            ref mut invalid_after,
        } = *self;
        let helper = UpdateHelper {
            old_lines: lines,
            old_invalid_before: invalid_before,
            old_invalid_after: invalid_after,
            new_lines: HashMap::new(),
            new_invalid_before: 0,
            new_invalid_after: 0,
        };
        helper.update(update.operations);
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

#[derive(Debug)]
struct UpdateHelper<'a, 'b, 'c> {
    old_lines: &'a mut HashMap<u64, Line>,
    old_invalid_before: &'b mut u64,
    old_invalid_after: &'c mut u64,
    new_lines: HashMap<u64, Line>,
    new_invalid_before: u64,
    new_invalid_after: u64,
}

impl<'a, 'b, 'c> UpdateHelper<'a, 'b, 'c> {
    fn apply_copy(&mut self, nb_lines: u64, first_line_num: Option<u64>) {
        debug!("copying {} lines", nb_lines);
        let UpdateHelper {
            ref mut old_lines,
            ref mut old_invalid_before,
            ref mut old_invalid_after,
            ref mut new_lines,
            ref mut new_invalid_before,
            ref mut new_invalid_after,
            ..
        } = *self;

        // The number of lines left to copy
        let mut nb_lines = nb_lines;

        // STEP 1: Handle the invalid lines that precede the valid ones
        // ------------------------------------------------------------

        if **old_invalid_before >= nb_lines {
            // case 1: there are more (or equal) invalid lines than lines to copy

            // decrement old_invalid_lines by nb_lines
            **old_invalid_before -= nb_lines;

            // and increment new_invalid_lines by the same amount
            *new_invalid_before += nb_lines;

            // there is no more line to copy so we're done
            return;
        } else if **old_invalid_after > 0 {
            // case 2: there are more lines to copy than invalid lines

            // decrement the nb of lines to copy by the number of invalid lines
            nb_lines -= **old_invalid_before;

            // increment new_invalid_lines by the same amount
            *new_invalid_before += **old_invalid_before;

            // we don't have any invalid lines left
            **old_invalid_before = 0;
        }

        // STEP 2: Handle the valid lines
        // ------------------------------------------------------------

        let nb_valid_lines = old_lines.len();
        let range;

        if nb_lines <= (nb_valid_lines as u64) {
            // case 1: the are more (or equal) valid lines than lines to copy

            // the range of lines to copy: from the start to nb_lines - 1;
            range = 0..nb_lines as usize;

            // after the copy, we won't have any line remaining to copy
            nb_lines = 0;
        } else {
            // case 2: there are more lines to copy than valid lines

            // we copy all the valid lines
            range = 0..nb_valid_lines;

            // after the operation we'll have (nb_lines - nb_valid_lines) left to copy
            nb_lines -= nb_valid_lines as u64;
        }

        // we'll only apply the copy if there actually are valid lines to copy
        if nb_valid_lines > 0 {
            let diff = if let Some(new_first_line_num) = first_line_num {
                // find the first "real" line (ie non-wrapped), and
                // compute the difference between its line number and
                // its *new* line number, given by the "copy"
                // operation. This will be used to update the line
                // number for all the lines we copy.
                let num = old_lines
                    .iter()
                    .filter_map(|(_, line)| line.line_num)
                    .min()
                    .unwrap_or(0);

                new_first_line_num as i64 - num as i64
            } else {
                // if the "copy" operation does not specify a new line
                // number, just set the diff to 0
                0
            };

            let copied_lines = range.map(|i| old_lines.remove_entry(&(i as u64)).unwrap()).map(|(i, mut line)| {
                line.line_num = line
                    .line_num
                    .map(|line_num| (line_num as i64 + diff) as u64);
                (i, line)
            });

            new_lines.extend(copied_lines);
        }

        // if there are no more lines to copy we're done
        if nb_lines == 0 {
            return;
        }

        // STEP 3: Handle the remaining invalid lines
        // ------------------------------------------------------------

        // We should have at least enough invalid lines to copy, otherwise it indicates there's a
        // problem, and we panic.
        if **old_invalid_after >= nb_lines {
            **old_invalid_after -= nb_lines;
            *new_invalid_after += nb_lines;
        } else {
            error!(
                "{} lines left to copy, but only {} lines in the old cache",
                nb_lines, **old_invalid_after
            );
            panic!("cache update failed");
        }
    }

    fn apply_skip(&mut self, nb_lines: u64) {
        debug!("skipping {} lines", nb_lines);

        let UpdateHelper {
            ref mut old_lines,
            ref mut old_invalid_before,
            ref mut old_invalid_after,
            ..
        } = *self;

        let mut nb_lines = nb_lines;

        // Skip invalid lines that comes before the valid ones.
        if **old_invalid_before > nb_lines {
            **old_invalid_before -= nb_lines;
            return;
        } else if **old_invalid_before > 0 {
            nb_lines -= **old_invalid_before;
            **old_invalid_before = 0;
        }

        // Skip the valid lines
        let nb_valid_lines = old_lines.len();
        if nb_lines < nb_valid_lines as u64 {
            let range = 0..nb_lines;
            range.map(|i| old_lines.remove(&i)).last();;
            return;
        } else {
            old_lines.clear();
            nb_lines -= nb_valid_lines as u64;
        }

        // Skip the remaining invalid lines
        if **old_invalid_after >= nb_lines {
            **old_invalid_after -= nb_lines;
            return;
        }

        error!(
            "{} lines left to skip, but only {} lines in the old cache",
            nb_lines, **old_invalid_after
        );
        panic!("cache update failed");
    }

    fn apply_invalidate(&mut self, nb_lines: u64) {
        debug!("invalidating {} lines", nb_lines);
        if self.new_lines.is_empty() {
            self.new_invalid_before += nb_lines;
        } else {
            self.new_invalid_after += nb_lines;
        }
    }

    fn apply_insert(&mut self, mut lines: Vec<Line>) {
        debug!("inserting {} lines", lines.len());
        // We need to insert at last line +1, but we still want to start counting from 0
        // for the first line.
        let mut last_line: i64 = if let Some(num) = self.new_lines.keys().max() {
            *num as i64
        } else {
            -1
        };

        self.new_lines.extend(lines.drain(..).map(|mut line| {
            trim_new_line(&mut line.text);
            last_line += 1;
            let ret = (last_line as u64, line);
            ret
        }));
    }

    fn apply_update(&mut self, nb_lines: u64, lines: Vec<Line>) {
        debug!("updating {} lines", nb_lines);
        let UpdateHelper {
            ref mut old_lines,
            ref mut new_lines,
            ..
        } = *self;
        if nb_lines > old_lines.len() as u64 {
            error!(
                "{} lines to update, but only {} lines in cache",
                nb_lines,
                old_lines.len()
            );
            panic!("failed to update the cache");
        }

        let range = 0..nb_lines;

        new_lines.extend(
            range
                .map(|i| old_lines.remove_entry(&i).unwrap())
                .zip(lines.into_iter())
                .map(|((i, mut old_line), update)| {
                    old_line.cursor = update.cursor;
                    old_line.styles = update.styles;
                    (i, old_line)
                }),
        )
    }

    fn update(mut self, operations: Vec<Operation>) {
        trace!("updating the line cache");
        trace!("cache state before: {:?}", &self);
        trace!("operations to be applied: {:?}", &operations);
        for op in operations {
            debug!("operation: {:?}", &op);
            debug!("cache helper before operation {:?}", &self);
            match op.operation_type {
                OperationType::Copy_ => (&mut self).apply_copy(op.nb_lines, op.line_num),
                OperationType::Skip => (&mut self).apply_skip(op.nb_lines),
                OperationType::Invalidate => (&mut self).apply_invalidate(op.nb_lines),
                OperationType::Insert => (&mut self).apply_insert(op.lines),
                OperationType::Update => (&mut self).apply_update(op.nb_lines, op.lines),
            }
            debug!("cache helper after operation {:?}", &self);
        }
        *self.old_lines = self.new_lines;
        *self.old_invalid_before = self.new_invalid_before;
        *self.old_invalid_after = self.new_invalid_after;
    }
}

fn trim_new_line(text: &mut String) {
    if let Some('\n') = text.chars().last() {
        text.pop();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{OperationType::*, Update, *};

    #[test]
    fn linecache_1() {
        let updates = vec![
            Update { rev: None, operations: [Operation { operation_type: Insert, nb_lines: 12, line_num: None, lines: [Line { text: "[package]\n".to_string(), cursor: [0].to_vec(), styles: [].to_vec(), line_num: Some(1) }, Line { text: "authors ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(2) }, Line { text: "= ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "[\"Corentin ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "Henry ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "<corentinhenry@gmail.com>\"]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "description ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(3) }, Line { text: "= ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "\"Xi ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "Rpc ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "Lib ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "- ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }].to_vec() }, Operation { operation_type: Invalidate, nb_lines: 115, line_num: None, lines: [].to_vec() }].to_vec(), pristine: true, view_id: ViewId(1) },
            Update { rev: None, operations: [Operation { operation_type: Insert, nb_lines: 12, line_num: None, lines: [Line { text: "[package]\n".to_string(), cursor: [0].to_vec(), styles: [].to_vec(), line_num: Some(1) }, Line { text: "authors ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(2) }, Line { text: "= ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "[\"Corentin ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "Henry ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "<corentinhenry@gmail.com>\"]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "description ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(3) }, Line { text: "= ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "\"Xi ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "Rpc ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "Lib ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "- ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }].to_vec() }, Operation { operation_type: Invalidate, nb_lines: 115, line_num: None, lines: [].to_vec() }].to_vec(), pristine: true, view_id: ViewId(1) },
            Update { rev: None, operations: [Operation { operation_type: Copy_, nb_lines: 12, line_num: Some(1), lines: [].to_vec() }, Operation { operation_type: Insert, nb_lines: 38, line_num: None, lines: [Line { text: "Tokio ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "based ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "implementation ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "of ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "the ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "RPC ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "used ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "in ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "the ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "Xi ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "editor\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "homepage ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(4) }, Line { text: "= ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "\"https://".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "github.com/".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "xi-".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "frontend/".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "xrl\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "keywords ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(5) }, Line { text: "= ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "[\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "    ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(6) }, Line { text: "\"xi\".to_string(),\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "    ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(7) }, Line { text: "\"rpc\".to_string(),\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "    ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(8) }, Line { text: "\"json-".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "rpc\".to_string(),\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(9) }, Line { text: "license-".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(10) }, Line { text: "file ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "= ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "\"LICENSE-".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "MIT\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "name ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(11) }, Line { text: "= ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "\"xrl\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "readme ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(12) }].to_vec() }, Operation { operation_type: Invalidate, nb_lines: 77, line_num: None, lines: [].to_vec() }].to_vec(), pristine: true, view_id: ViewId(1) },
            Update { rev: None, operations: [Operation { operation_type: Insert, nb_lines: 38, line_num: None, lines: [Line { text: "[package]\n".to_string(), cursor: [0].to_vec(), styles: [].to_vec(), line_num: Some(1) }, Line { text: "authors = [\"Corentin Henry <corentinhenry@gmail.com>\"]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(2) }, Line { text: "description = \"Xi Rpc Lib - Tokio based implementation of the RPC used in the Xi editor\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(3) }, Line { text: "homepage = \"https://github.com/xi-frontend/xrl\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(4) }, Line { text: "keywords = [\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(5) }, Line { text: "    \"xi\".to_string(),\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(6) }, Line { text: "    \"rpc\".to_string(),\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(7) }, Line { text: "    \"json-rpc\".to_string(),\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(8) }, Line { text: "]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(9) }, Line { text: "license-file = \"LICENSE-MIT\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(10) }, Line { text: "name = \"xrl\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(11) }, Line { text: "readme = \"README.md\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(12) }, Line { text: "repository = \"https://github.com/xi-frontend/xrl\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(13) }, Line { text: "version = \"0.0.6\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(14) }, Line { text: "edition = \"2018\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(15) }, Line { text: "\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(16) }, Line { text: "[dependencies]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(17) }, Line { text: "bytes = \"0.4.12\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(18) }, Line { text: "futures = \"0.1.27\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(19) }, Line { text: "log = \"0.4.6\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(20) }, Line { text: "serde = \"1.0.92\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(21) }, Line { text: "serde_derive = \"1.0.92\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(22) }, Line { text: "serde_json = \"1.0.39\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(23) }, Line { text: "tokio = \"0.1.21\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(24) }, Line { text: "tokio-codec = \"0.1.1\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(25) }, Line { text: "tokio-process = \"0.2.3\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(26) }, Line { text: "syntect = { version = \"3.2.0\".to_string(), default-features = false }\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(27) }, Line { text: "\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(28) }, Line { text: "[dependencies.clippy]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(29) }, Line { text: "optional = true\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(30) }, Line { text: "version = \"0.0.302\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(31) }, Line { text: "\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(32) }, Line { text: "[dev-dependencies]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(33) }, Line { text: "criterion = \"0.2\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(34) }, Line { text: "\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(35) }, Line { text: "[[bench]]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(36) }, Line { text: "name = \"linecache\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(37) }, Line { text: "harness = false".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(38) }].to_vec() }].to_vec(), pristine: true, view_id: ViewId(1) },
            Update { rev: None, operations: [Operation { operation_type: Insert, nb_lines: 38, line_num: None, lines: [Line { text: "[package]\n".to_string(), cursor: [0].to_vec(), styles: [].to_vec(), line_num: Some(1) }, Line { text: "authors = [\"Corentin Henry <corentinhenry@gmail.com>\"]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(2) }, Line { text: "description = \"Xi Rpc Lib - Tokio based implementation of the RPC used in the Xi editor\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(3) }, Line { text: "homepage = \"https://github.com/xi-frontend/xrl\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(4) }, Line { text: "keywords = [\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(5) }, Line { text: "    \"xi\".to_string(),\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(6) }, Line { text: "    \"rpc\".to_string(),\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(7) }, Line { text: "    \"json-rpc\".to_string(),\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(8) }, Line { text: "]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(9) }, Line { text: "license-file = \"LICENSE-MIT\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(10) }, Line { text: "name = \"xrl\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(11) }, Line { text: "readme = \"README.md\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(12) }, Line { text: "repository = \"https://github.com/xi-frontend/xrl\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(13) }, Line { text: "version = \"0.0.6\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(14) }, Line { text: "edition = \"2018\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(15) }, Line { text: "\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(16) }, Line { text: "[dependencies]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(17) }, Line { text: "bytes = \"0.4.12\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(18) }, Line { text: "futures = \"0.1.27\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(19) }, Line { text: "log = \"0.4.6\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(20) }, Line { text: "serde = \"1.0.92\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(21) }, Line { text: "serde_derive = \"1.0.92\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(22) }, Line { text: "serde_json = \"1.0.39\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(23) }, Line { text: "tokio = \"0.1.21\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(24) }, Line { text: "tokio-codec = \"0.1.1\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(25) }, Line { text: "tokio-process = \"0.2.3\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(26) }, Line { text: "syntect = { version = \"3.2.0\".to_string(), default-features = false }\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(27) }, Line { text: "\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(28) }, Line { text: "[dependencies.clippy]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(29) }, Line { text: "optional = true\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(30) }, Line { text: "version = \"0.0.302\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(31) }, Line { text: "\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(32) }, Line { text: "[dev-dependencies]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(33) }, Line { text: "criterion = \"0.2\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(34) }, Line { text: "\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(35) }, Line { text: "[[bench]]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(36) }, Line { text: "name = \"linecache\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(37) }, Line { text: "harness = false".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(38) }].to_vec() }].to_vec(), pristine: true, view_id: ViewId(1) },
            Update { rev: None, operations: [Operation { operation_type: Insert, nb_lines: 38, line_num: None, lines: [Line { text: "[package]\n".to_string(), cursor: [0].to_vec(), styles: [].to_vec(), line_num: Some(1) }, Line { text: "authors = [\"Corentin Henry <corentinhenry@gmail.com>\"]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(2) }, Line { text: "description = \"Xi Rpc Lib - Tokio based implementation of the RPC used in the Xi editor\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(3) }, Line { text: "homepage = \"https://github.com/xi-frontend/xrl\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(4) }, Line { text: "keywords = [\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(5) }, Line { text: "    \"xi\".to_string(),\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(6) }, Line { text: "    \"rpc\".to_string(),\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(7) }, Line { text: "    \"json-rpc\".to_string(),\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(8) }, Line { text: "]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(9) }, Line { text: "license-file = \"LICENSE-MIT\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(10) }, Line { text: "name = \"xrl\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(11) }, Line { text: "readme = \"README.md\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(12) }, Line { text: "repository = \"https://github.com/xi-frontend/xrl\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(13) }, Line { text: "version = \"0.0.6\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(14) }, Line { text: "edition = \"2018\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(15) }, Line { text: "\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(16) }, Line { text: "[dependencies]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(17) }, Line { text: "bytes = \"0.4.12\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(18) }, Line { text: "futures = \"0.1.27\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(19) }, Line { text: "log = \"0.4.6\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(20) }, Line { text: "serde = \"1.0.92\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(21) }, Line { text: "serde_derive = \"1.0.92\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(22) }, Line { text: "serde_json = \"1.0.39\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(23) }, Line { text: "tokio = \"0.1.21\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(24) }, Line { text: "tokio-codec = \"0.1.1\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(25) }, Line { text: "tokio-process = \"0.2.3\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(26) }, Line { text: "syntect = { version = \"3.2.0\".to_string(), default-features = false }\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(27) }, Line { text: "\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(28) }, Line { text: "[dependencies.clippy]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(29) }, Line { text: "optional = true\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(30) }, Line { text: "version = \"0.0.302\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(31) }, Line { text: "\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(32) }, Line { text: "[dev-dependencies]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(33) }, Line { text: "criterion = \"0.2\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(34) }, Line { text: "\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(35) }, Line { text: "[[bench]]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(36) }, Line { text: "name = \"linecache\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(37) }, Line { text: "harness = false".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(38) }].to_vec() }].to_vec(), pristine: true, view_id: ViewId(1) },
            Update { rev: None, operations: [Operation { operation_type: Insert, nb_lines: 38, line_num: None, lines: [Line { text: "[package]\n".to_string(), cursor: [0].to_vec(), styles: [].to_vec(), line_num: Some(1) }, Line { text: "authors = [\"Corentin Henry <corentinhenry@gmail.com>\"]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(2) }, Line { text: "description = \"Xi Rpc Lib - Tokio based implementation of the RPC used in the Xi editor\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(3) }, Line { text: "homepage = \"https://github.com/xi-frontend/xrl\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(4) }, Line { text: "keywords = [\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(5) }, Line { text: "    \"xi\".to_string(),\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(6) }, Line { text: "    \"rpc\".to_string(),\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(7) }, Line { text: "    \"json-rpc\".to_string(),\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(8) }, Line { text: "]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(9) }, Line { text: "license-file = \"LICENSE-MIT\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(10) }, Line { text: "name = \"xrl\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(11) }, Line { text: "readme = \"README.md\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(12) }, Line { text: "repository = \"https://github.com/xi-frontend/xrl\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(13) }, Line { text: "version = \"0.0.6\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(14) }, Line { text: "edition = \"2018\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(15) }, Line { text: "\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(16) }, Line { text: "[dependencies]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(17) }, Line { text: "bytes = \"0.4.12\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(18) }, Line { text: "futures = \"0.1.27\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(19) }, Line { text: "log = \"0.4.6\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(20) }, Line { text: "serde = \"1.0.92\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(21) }, Line { text: "serde_derive = \"1.0.92\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(22) }, Line { text: "serde_json = \"1.0.39\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(23) }, Line { text: "tokio = \"0.1.21\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(24) }, Line { text: "tokio-codec = \"0.1.1\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(25) }, Line { text: "tokio-process = \"0.2.3\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(26) }, Line { text: "syntect = { version = \"3.2.0\".to_string(), default-features = false }\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(27) }, Line { text: "\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(28) }, Line { text: "[dependencies.clippy]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(29) }, Line { text: "optional = true\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(30) }, Line { text: "version = \"0.0.302\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(31) }, Line { text: "\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(32) }, Line { text: "[dev-dependencies]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(33) }, Line { text: "criterion = \"0.2\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(34) }, Line { text: "\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(35) }, Line { text: "[[bench]]\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(36) }, Line { text: "name = \"linecache\"\n".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(37) }, Line { text: "harness = false".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(38) }].to_vec() }].to_vec(), pristine: true, view_id: ViewId(1) }
        ];

        let mut lines = vec![
            [Line { text: "[package]".to_string(), cursor: [0].to_vec(), styles: [].to_vec(), line_num: Some(1) }, Line { text: "authors ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(2) }, Line { text: "= ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "[\"Corentin ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "Henry ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "<corentinhenry@gmail.com>\"]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "description ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(3) }, Line { text: "= ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "\"Xi ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "Rpc ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "Lib ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "- ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }].to_vec(),
            [Line { text: "[package]".to_string(), cursor: [0].to_vec(), styles: [].to_vec(), line_num: Some(1) }, Line { text: "authors ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(2) }, Line { text: "= ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "[\"Corentin ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "Henry ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "<corentinhenry@gmail.com>\"]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "description ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(3) }, Line { text: "= ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "\"Xi ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "Rpc ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "Lib ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "- ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }].to_vec(),
            [Line { text: "[package]".to_string(), cursor: [0].to_vec(), styles: [].to_vec(), line_num: Some(1) }, Line { text: "authors ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(2) }, Line { text: "= ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "[\"Corentin ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "Henry ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "<corentinhenry@gmail.com>\"]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "description ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(3) }, Line { text: "= ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "\"Xi ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "Rpc ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "Lib ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "- ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "Tokio ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "based ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "implementation ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "of ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "the ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "RPC ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "used ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "in ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "the ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "Xi ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "editor\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "homepage ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(4) }, Line { text: "= ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "\"https://".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "github.com/".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "xi-".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "frontend/".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "xrl\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "keywords ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(5) }, Line { text: "= ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "[".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "    ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(6) }, Line { text: "\"xi\".to_string(),".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "    ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(7) }, Line { text: "\"rpc\".to_string(),".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "    ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(8) }, Line { text: "\"json-".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "rpc\".to_string(),".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(9) }, Line { text: "license-".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(10) }, Line { text: "file ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "= ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "\"LICENSE-".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "MIT\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "name ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(11) }, Line { text: "= ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "\"xrl\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: None }, Line { text: "readme ".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(12) }].to_vec(),
            [Line { text: "[package]".to_string(), cursor: [0].to_vec(), styles: [].to_vec(), line_num: Some(1) }, Line { text: "authors = [\"Corentin Henry <corentinhenry@gmail.com>\"]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(2) }, Line { text: "description = \"Xi Rpc Lib - Tokio based implementation of the RPC used in the Xi editor\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(3) }, Line { text: "homepage = \"https://github.com/xi-frontend/xrl\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(4) }, Line { text: "keywords = [".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(5) }, Line { text: "    \"xi\".to_string(),".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(6) }, Line { text: "    \"rpc\".to_string(),".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(7) }, Line { text: "    \"json-rpc\".to_string(),".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(8) }, Line { text: "]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(9) }, Line { text: "license-file = \"LICENSE-MIT\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(10) }, Line { text: "name = \"xrl\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(11) }, Line { text: "readme = \"README.md\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(12) }, Line { text: "repository = \"https://github.com/xi-frontend/xrl\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(13) }, Line { text: "version = \"0.0.6\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(14) }, Line { text: "edition = \"2018\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(15) }, Line { text: "".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(16) }, Line { text: "[dependencies]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(17) }, Line { text: "bytes = \"0.4.12\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(18) }, Line { text: "futures = \"0.1.27\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(19) }, Line { text: "log = \"0.4.6\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(20) }, Line { text: "serde = \"1.0.92\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(21) }, Line { text: "serde_derive = \"1.0.92\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(22) }, Line { text: "serde_json = \"1.0.39\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(23) }, Line { text: "tokio = \"0.1.21\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(24) }, Line { text: "tokio-codec = \"0.1.1\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(25) }, Line { text: "tokio-process = \"0.2.3\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(26) }, Line { text: "syntect = { version = \"3.2.0\".to_string(), default-features = false }".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(27) }, Line { text: "".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(28) }, Line { text: "[dependencies.clippy]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(29) }, Line { text: "optional = true".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(30) }, Line { text: "version = \"0.0.302\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(31) }, Line { text: "".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(32) }, Line { text: "[dev-dependencies]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(33) }, Line { text: "criterion = \"0.2\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(34) }, Line { text: "".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(35) }, Line { text: "[[bench]]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(36) }, Line { text: "name = \"linecache\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(37) }, Line { text: "harness = false".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(38) }].to_vec(),
            [Line { text: "[package]".to_string(), cursor: [0].to_vec(), styles: [].to_vec(), line_num: Some(1) }, Line { text: "authors = [\"Corentin Henry <corentinhenry@gmail.com>\"]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(2) }, Line { text: "description = \"Xi Rpc Lib - Tokio based implementation of the RPC used in the Xi editor\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(3) }, Line { text: "homepage = \"https://github.com/xi-frontend/xrl\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(4) }, Line { text: "keywords = [".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(5) }, Line { text: "    \"xi\".to_string(),".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(6) }, Line { text: "    \"rpc\".to_string(),".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(7) }, Line { text: "    \"json-rpc\".to_string(),".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(8) }, Line { text: "]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(9) }, Line { text: "license-file = \"LICENSE-MIT\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(10) }, Line { text: "name = \"xrl\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(11) }, Line { text: "readme = \"README.md\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(12) }, Line { text: "repository = \"https://github.com/xi-frontend/xrl\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(13) }, Line { text: "version = \"0.0.6\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(14) }, Line { text: "edition = \"2018\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(15) }, Line { text: "".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(16) }, Line { text: "[dependencies]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(17) }, Line { text: "bytes = \"0.4.12\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(18) }, Line { text: "futures = \"0.1.27\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(19) }, Line { text: "log = \"0.4.6\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(20) }, Line { text: "serde = \"1.0.92\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(21) }, Line { text: "serde_derive = \"1.0.92\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(22) }, Line { text: "serde_json = \"1.0.39\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(23) }, Line { text: "tokio = \"0.1.21\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(24) }, Line { text: "tokio-codec = \"0.1.1\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(25) }, Line { text: "tokio-process = \"0.2.3\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(26) }, Line { text: "syntect = { version = \"3.2.0\".to_string(), default-features = false }".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(27) }, Line { text: "".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(28) }, Line { text: "[dependencies.clippy]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(29) }, Line { text: "optional = true".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(30) }, Line { text: "version = \"0.0.302\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(31) }, Line { text: "".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(32) }, Line { text: "[dev-dependencies]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(33) }, Line { text: "criterion = \"0.2\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(34) }, Line { text: "".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(35) }, Line { text: "[[bench]]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(36) }, Line { text: "name = \"linecache\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(37) }, Line { text: "harness = false".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(38) }].to_vec(),
            [Line { text: "[package]".to_string(), cursor: [0].to_vec(), styles: [].to_vec(), line_num: Some(1) }, Line { text: "authors = [\"Corentin Henry <corentinhenry@gmail.com>\"]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(2) }, Line { text: "description = \"Xi Rpc Lib - Tokio based implementation of the RPC used in the Xi editor\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(3) }, Line { text: "homepage = \"https://github.com/xi-frontend/xrl\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(4) }, Line { text: "keywords = [".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(5) }, Line { text: "    \"xi\".to_string(),".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(6) }, Line { text: "    \"rpc\".to_string(),".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(7) }, Line { text: "    \"json-rpc\".to_string(),".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(8) }, Line { text: "]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(9) }, Line { text: "license-file = \"LICENSE-MIT\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(10) }, Line { text: "name = \"xrl\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(11) }, Line { text: "readme = \"README.md\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(12) }, Line { text: "repository = \"https://github.com/xi-frontend/xrl\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(13) }, Line { text: "version = \"0.0.6\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(14) }, Line { text: "edition = \"2018\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(15) }, Line { text: "".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(16) }, Line { text: "[dependencies]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(17) }, Line { text: "bytes = \"0.4.12\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(18) }, Line { text: "futures = \"0.1.27\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(19) }, Line { text: "log = \"0.4.6\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(20) }, Line { text: "serde = \"1.0.92\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(21) }, Line { text: "serde_derive = \"1.0.92\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(22) }, Line { text: "serde_json = \"1.0.39\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(23) }, Line { text: "tokio = \"0.1.21\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(24) }, Line { text: "tokio-codec = \"0.1.1\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(25) }, Line { text: "tokio-process = \"0.2.3\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(26) }, Line { text: "syntect = { version = \"3.2.0\".to_string(), default-features = false }".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(27) }, Line { text: "".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(28) }, Line { text: "[dependencies.clippy]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(29) }, Line { text: "optional = true".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(30) }, Line { text: "version = \"0.0.302\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(31) }, Line { text: "".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(32) }, Line { text: "[dev-dependencies]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(33) }, Line { text: "criterion = \"0.2\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(34) }, Line { text: "".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(35) }, Line { text: "[[bench]]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(36) }, Line { text: "name = \"linecache\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(37) }, Line { text: "harness = false".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(38) }].to_vec(),
            [Line { text: "[package]".to_string(), cursor: [0].to_vec(), styles: [].to_vec(), line_num: Some(1) }, Line { text: "authors = [\"Corentin Henry <corentinhenry@gmail.com>\"]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(2) }, Line { text: "description = \"Xi Rpc Lib - Tokio based implementation of the RPC used in the Xi editor\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(3) }, Line { text: "homepage = \"https://github.com/xi-frontend/xrl\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(4) }, Line { text: "keywords = [".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(5) }, Line { text: "    \"xi\".to_string(),".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(6) }, Line { text: "    \"rpc\".to_string(),".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(7) }, Line { text: "    \"json-rpc\".to_string(),".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(8) }, Line { text: "]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(9) }, Line { text: "license-file = \"LICENSE-MIT\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(10) }, Line { text: "name = \"xrl\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(11) }, Line { text: "readme = \"README.md\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(12) }, Line { text: "repository = \"https://github.com/xi-frontend/xrl\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(13) }, Line { text: "version = \"0.0.6\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(14) }, Line { text: "edition = \"2018\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(15) }, Line { text: "".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(16) }, Line { text: "[dependencies]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(17) }, Line { text: "bytes = \"0.4.12\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(18) }, Line { text: "futures = \"0.1.27\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(19) }, Line { text: "log = \"0.4.6\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(20) }, Line { text: "serde = \"1.0.92\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(21) }, Line { text: "serde_derive = \"1.0.92\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(22) }, Line { text: "serde_json = \"1.0.39\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(23) }, Line { text: "tokio = \"0.1.21\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(24) }, Line { text: "tokio-codec = \"0.1.1\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(25) }, Line { text: "tokio-process = \"0.2.3\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(26) }, Line { text: "syntect = { version = \"3.2.0\".to_string(), default-features = false }".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(27) }, Line { text: "".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(28) }, Line { text: "[dependencies.clippy]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(29) }, Line { text: "optional = true".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(30) }, Line { text: "version = \"0.0.302\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(31) }, Line { text: "".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(32) }, Line { text: "[dev-dependencies]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(33) }, Line { text: "criterion = \"0.2\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(34) }, Line { text: "".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(35) }, Line { text: "[[bench]]".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(36) }, Line { text: "name = \"linecache\"".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(37) }, Line { text: "harness = false".to_string(), cursor: [].to_vec(), styles: [].to_vec(), line_num: Some(38) }].to_vec(),
        ];

        let line_cache_states = vec![
            LineCache { invalid_before: 0, lines: lines[0].drain(..).enumerate().map(|(u, l)| (u as u64, l)).collect(), invalid_after: 115 },
            LineCache { invalid_before: 0, lines: lines[1].drain(..).enumerate().map(|(u, l)| (u as u64, l)).collect(), invalid_after: 115 },
            LineCache { invalid_before: 0, lines: lines[2].drain(..).enumerate().map(|(u, l)| (u as u64, l)).collect(), invalid_after: 77 },
            LineCache { invalid_before: 0, lines: lines[3].drain(..).enumerate().map(|(u, l)| (u as u64, l)).collect(), invalid_after: 0 },
            LineCache { invalid_before: 0, lines: lines[4].drain(..).enumerate().map(|(u, l)| (u as u64, l)).collect(), invalid_after: 0 },
            LineCache { invalid_before: 0, lines: lines[5].drain(..).enumerate().map(|(u, l)| (u as u64, l)).collect(), invalid_after: 0 },
            LineCache { invalid_before: 0, lines: lines[6].drain(..).enumerate().map(|(u, l)| (u as u64, l)).collect(), invalid_after: 0 },
        ];

        let mut linecache = LineCache::default();

        for (i, up) in updates.iter().enumerate() {
            linecache.update(up.clone());
            println!("Checking linecache for iteration {}", i);
            assert_eq!(linecache, line_cache_states[i]);
        }
    }
}