use crate::strategies::{Builder, Slot, Strategy, StrategyMatch};

/// This strategy tries to find the most specific match of a domain respecting the domain hierarchy.
///
/// For example take the following two domains we want to match for:
/// - `www.example.com`
/// - `example.com`
///
/// Expected behavior:
/// ```not_rust
/// Input -> Match
///
/// beispiel.de -> N/A
/// example.com -> example.com
/// subdomain.example.com -> example.com
/// www.example.com -> www.example.com
/// ```
pub struct DomainLevelsStrategy {
    // aho corasick is a magical algorithm that lets is search a byte sequence (string) for
    // a bundle of keywords in a very efficient way
    matcher: aho_corasick::AhoCorasick,
    table: Vec<Slot>,
}

#[derive(Default)]
pub struct DomainLevelsStrategyBuilder;

impl Strategy for DomainLevelsStrategy {
    type Builder = DomainLevelsStrategyBuilder;

    fn r#match(&self, string: &str) -> Option<StrategyMatch> {
        self.matcher
            .find_iter(string)
            // The matches must be aligned to the end of the domain
            .filter(|m| m.end() == string.len())
            // The match may not be inside of the domain, only two cases are valid:
            // 1. The match is at the beginning of the domain, meaning that it is a exact match
            // 2. The match is aligned to a part of the domain, so a dot is on the left of the match
            .filter(|m| m.start() == 0 || string.as_bytes()[m.start() - 1] == b'.')
            // Find the longest match as it is the most specific
            .min_by(|x, y| x.start().cmp(&y.start()))
            .map(|m| StrategyMatch {
                slot: self.table[m.pattern()],
                any: None,
            })
    }
}

impl Builder for DomainLevelsStrategyBuilder {
    type Strategy = DomainLevelsStrategy;
    type Error = ();

    fn build(self) -> Result<Self::Strategy, Self::Error> {
        todo!()
    }

    fn add(&mut self, string: &str, slot: Slot) -> &mut Self {
        todo!()
    }
}
