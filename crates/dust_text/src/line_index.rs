use crate::{TextRange, TextSize};

/// A zero-based line and column pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineCol {
    /// Zero-based line number.
    pub line: u32,
    /// Zero-based byte column within that line.
    pub column: u32,
}

/// Maps byte offsets to line/column locations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineIndex {
    line_starts: Vec<TextSize>,
    text_len: TextSize,
}

impl LineIndex {
    /// Builds an index for one source string.
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![TextSize::new(0)];
        for (offset, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(TextSize::from(offset + 1));
            }
        }

        Self {
            line_starts,
            text_len: TextSize::from(source.len()),
        }
    }

    /// Returns the start offset of every line.
    pub fn line_starts(&self) -> &[TextSize] {
        &self.line_starts
    }

    /// Returns the number of lines in the indexed source.
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Returns the start offset of one line.
    pub fn line_start(&self, line: usize) -> Option<TextSize> {
        self.line_starts.get(line).copied()
    }

    /// Converts one byte offset into a zero-based line/column pair.
    pub fn line_col(&self, offset: impl Into<TextSize>) -> Option<LineCol> {
        let offset = offset.into();
        if offset > self.text_len {
            return None;
        }

        let line = self
            .line_starts
            .partition_point(|&start| start <= offset)
            .checked_sub(1)?;
        let line_start = self.line_starts[line];

        Some(LineCol {
            line: line as u32,
            column: (offset - line_start).to_u32(),
        })
    }

    /// Returns the byte range covered by one line, excluding the trailing newline.
    pub fn line_range(&self, line: usize) -> Option<TextRange> {
        let start = *self.line_starts.get(line)?;
        let next = self
            .line_starts
            .get(line + 1)
            .copied()
            .unwrap_or(self.text_len);

        let end = if next > start {
            let last = next - TextSize::new(1);
            if last >= start && self.line_col(last).is_some() {
                if last == start || self.contains_newline(last) {
                    last
                } else {
                    next
                }
            } else {
                next
            }
        } else {
            next
        };

        let end = if end > start && self.line_starts.contains(&end) {
            end
        } else {
            next
        };

        Some(TextRange::new(start, end))
    }

    fn contains_newline(&self, offset: TextSize) -> bool {
        self.line_starts.binary_search(&offset).is_ok()
    }
}
