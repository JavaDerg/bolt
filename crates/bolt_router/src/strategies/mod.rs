use regex::Error;

mod exact;
mod pattern;
mod domain_levels;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Slot(pub(crate) u64);

pub trait Strategy: 'static + Sync {
    fn r#match(&self, string: &str) -> Option<Slot>;
}

pub trait Builder {
    type Strategy: Strategy;
    type Error;

    fn build(self) -> Result<Self::Strategy, Error>;
    // Input is expected to be normalized and NOT url-encoded
    fn add(&mut self, string: &str, slot: Slot) -> &mut Self;
    fn add_unnormalized(&mut self, segment: &str, slot: Slot) -> &mut Self {
        let normalized = bolt_url::normalize_str(segment);
        self.add(&normalized, slot)
    }
}
