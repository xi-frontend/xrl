use crate::{Line, Operation, OperationType, Update};

/// Line cache struct to work with xi update protocol.
#[derive(Clone, Debug, Default)]
pub struct LineCache {
    invalid_before: u64,
    lines: Vec<Line>,
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
    pub fn lines(&self) -> &Vec<Line> {
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
            new_lines: Vec::new(),
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
    old_lines: &'a mut Vec<Line>,
    old_invalid_before: &'b mut u64,
    old_invalid_after: &'c mut u64,
    new_lines: Vec<Line>,
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
                old_lines
                    .iter()
                    .find_map(|line| {
                        line.line_num
                            .map(|num| new_first_line_num as i64 - num as i64)
                    })
                    .unwrap_or(0)
            } else {
                // if the "copy" operation does not specify a new line
                // number, just set the diff to 0
                0
            };

            let copied_lines = old_lines.drain(range).map(|mut line| {
                line.line_num = line
                    .line_num
                    .map(|line_num| (line_num as i64 + diff) as u64);
                line
            });
            new_lines.extend(copied_lines);
        }

        // if there are no more lines to copy we're done
        if nb_lines == 0 {
            return;
        }

        // STEP 3: Handle the remaining invalid lines
        // ------------------------------------------------------------

        // We should have at least enought invalid lines to copy, otherwise it indicates there's a
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
            old_lines.drain(0..nb_lines as usize).last();
            return;
        } else {
            old_lines.drain(..).last();
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
        self.new_lines.extend(lines.drain(..).map(|mut line| {
            trim_new_line(&mut line.text);
            line
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
        new_lines.extend(
            old_lines
                .drain(0..nb_lines as usize)
                .zip(lines.into_iter())
                .map(|(mut old_line, update)| {
                    old_line.cursor = update.cursor;
                    old_line.styles = update.styles;
                    old_line
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
