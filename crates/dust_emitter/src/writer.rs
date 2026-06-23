/// A small indentation-aware writer for deterministic Dart source emission.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct DartWriter {
    /// Accumulated source buffer.
    buffer: String,
    /// Current indentation level in two-space units.
    indent: usize,
}

impl DartWriter {
    /// Creates a new empty writer.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Writes one logical line using the current indentation level.
    pub(crate) fn line(&mut self, line: impl AsRef<str>) {
        self.write_indented_line(line.as_ref());
    }

    /// Writes one blank line unless the current output already ends in one.
    pub(crate) fn blank_line(&mut self) {
        if self.buffer.is_empty() || self.buffer.ends_with("\n\n") {
            return;
        }

        self.buffer.push('\n');
    }

    /// Writes one multi-line block using the current indentation level.
    pub(crate) fn raw_block(&mut self, source: &str) {
        for line in source.lines() {
            if line.is_empty() {
                self.buffer.push('\n');
            } else {
                self.write_indented_line(line);
            }
        }
    }

    /// Finalizes the output buffer.
    pub(crate) fn finish(self) -> String {
        self.buffer
    }

    /// Writes one line with the current indentation level applied.
    fn write_indented_line(&mut self, line: &str) {
        self.buffer.push_str(&" ".repeat(self.indent * 2));
        self.buffer.push_str(line);
        self.buffer.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::DartWriter;

    #[test]
    fn writer_renders_lines_with_stable_spacing() {
        let mut writer = DartWriter::new();
        writer.line("// GENERATED");
        writer.blank_line();
        writer.line("@override");
        writer.line("String toString() => 'User()';");

        assert_eq!(
            writer.finish(),
            "// GENERATED\n\n@override\nString toString() => 'User()';\n"
        );
    }

    #[test]
    fn writer_raw_block_preserves_internal_blank_lines() {
        let mut writer = DartWriter::new();
        writer.raw_block(
            "@override\nString toString() => 'User()';\n\n@override\nint get hashCode => 1;",
        );

        assert_eq!(
            writer.finish(),
            "@override\nString toString() => 'User()';\n\n@override\nint get hashCode => 1;\n"
        );
    }
}
