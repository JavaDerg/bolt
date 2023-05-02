use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind, StartKind};

pub struct PrefixMatcher<const SEP: char> {
    aho_corasick: AhoCorasick,
    lookup: Box<[usize]>,
}

pub struct PrefixMatch {
    pub index: usize,
    pub end: usize,
}

impl<const SEP: char> PrefixMatcher<SEP> {
    /// trailing separators will be removed from the prefixes, unless it is more than one separator
    pub fn new(mut prefixes: Vec<String>, lookup: Vec<usize>) -> Result<Self, aho_corasick::BuildError> {
        assert_eq!(prefixes.len(), lookup.len());

        prefixes
            .iter_mut()
            .filter(|x| !x.is_empty() && x.ends_with(SEP) && x.len() > 1)
            .filter(|x| {
                let mut crs = x.chars().rev();
                match (crs.next(), crs.next()) {
                    (Some('/'), Some('/')) => false,
                    (Some('/'), _) => true,
                    _ => false,
                }
            })
            .for_each(|x| {
                let _ = x.pop();
            });

        Ok(Self {
            aho_corasick: AhoCorasickBuilder::new()
                .start_kind(StartKind::Anchored)
                .build(prefixes)?
            lookup: lookup.into_boxed_slice(),
        })
    }

    pub fn match_(&self, input: &str) -> Option<PrefixMatch> {
        self.aho_corasick
            .find_iter(input)
            .filter(|m| m.end() == input.len() || input[m.end()..].starts_with(SEP))
            .map(|m| PrefixMatch {
                index: self.lookup[m.pattern()],
                end: m.end(),
            })
            .next()
    }
}
