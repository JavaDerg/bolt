use regex::RegexSet;

pub struct RegexMatcher {
    set: RegexSet,
    lookup: Box<[usize]>,
}

impl RegexMatcher {
    pub fn new(patterns: Vec<String>, lookup: Vec<usize>) -> Result<Self, regex::Error> {
        assert_eq!(patterns.len(), lookup.len());

        Ok(Self {
            set: RegexSet::new(patterns)?,
            lookup: lookup.into_boxed_slice(),
        })
    }

    pub fn match_(&self, input: &str) -> Option<usize> {
        self.set
            .matches(input)
            .into_iter()
            .next()
            .map(|i| self.lookup[i])
    }
}
