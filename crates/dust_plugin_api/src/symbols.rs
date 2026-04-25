/// One requested generated symbol name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestedSymbol {
    /// The reserved symbol name.
    pub name: String,
}

impl RequestedSymbol {
    /// Creates one requested symbol.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// The deterministic set of reserved generated symbols for one library.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SymbolPlan {
    reserved: Vec<RequestedSymbol>,
}

impl SymbolPlan {
    /// Reserves one symbol, keeping the first-seen order and ignoring duplicates.
    pub fn reserve(&mut self, name: impl Into<String>) {
        let symbol = RequestedSymbol::new(name);
        if !self.reserved.iter().any(|entry| entry.name == symbol.name) {
            self.reserved.push(symbol);
        }
    }

    /// Returns the reserved symbols in deterministic order.
    pub fn reserved(&self) -> &[RequestedSymbol] {
        &self.reserved
    }

    /// Returns `true` if the plan already contains the given symbol name.
    pub fn contains(&self, name: &str) -> bool {
        self.reserved.iter().any(|entry| entry.name == name)
    }
}
