use crate::DartFileIr;

/// A collection of lowered libraries.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkspaceIr {
    /// The lowered libraries in workspace order.
    pub libraries: Vec<DartFileIr>,
}

impl WorkspaceIr {
    /// Appends one lowered library.
    pub fn push_library(&mut self, library: DartFileIr) {
        self.libraries.push(library);
    }
}
