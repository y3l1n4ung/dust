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
    known_copyable_types: Vec<String>,
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

    /// Records one known `copyWith()`-capable Dart type name for emission decisions.
    pub fn add_copyable_type(&mut self, name: impl Into<String>) {
        let name = name.into();
        if !self.known_copyable_types.iter().any(|entry| entry == &name) {
            self.known_copyable_types.push(name);
        }
    }

    /// Extends the plan with multiple known `copyWith()`-capable Dart type names.
    pub fn extend_copyable_types<I>(&mut self, names: I)
    where
        I: IntoIterator,
        I::Item: Into<String>,
    {
        for name in names {
            self.add_copyable_type(name);
        }
    }

    /// Returns the known `copyWith()`-capable Dart type names in deterministic order.
    pub fn known_copyable_types(&self) -> &[String] {
        &self.known_copyable_types
    }

    /// Returns `true` if the plan already knows the given copyable type name.
    pub fn contains_copyable_type(&self, name: &str) -> bool {
        self.known_copyable_types.iter().any(|entry| entry == name)
    }
}
