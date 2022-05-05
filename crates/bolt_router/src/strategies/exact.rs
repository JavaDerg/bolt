use crate::strategies::{Builder, Slot, Strategy};

pub struct ExactStrategy {
    table: ahash::AHashMap<String, Slot>,
}

#[derive(Default)]
pub struct ExactStrategyBuilder {
    table: ahash::AHashMap<String, Slot>,
}

impl Strategy for ExactStrategy {
    type Builder = ExactStrategyBuilder;

    fn r#match(&self, segment: &str) -> Option<Slot> {
        self.table.get(segment).cloned()
    }
}

impl Builder for ExactStrategyBuilder {
    type Strategy = ExactStrategy;
    type Error = ();

    fn build(self) -> Result<Self::Strategy, Self::Error> {
        Ok(ExactStrategy { table: self.table })
    }

    fn add(&mut self, segment: &str, slot: Slot) -> &mut Self {
        self.table.insert(segment.to_string(), slot);
        self
    }
}
