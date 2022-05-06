use crate::strategies::{Builder, Slot, Strategy};

pub struct RegexStrategy {
    regex_set: regex::RegexSet,
    regexes: Vec<regex::Regex>,
    table: Vec<Slot>,
}

#[derive(Default)]
pub struct RegexStrategyBuilder {
    patterns: Vec<String>,
    table: Vec<Slot>,
}

impl Strategy for RegexStrategy {
    type Builder = RegexStrategyBuilder;

    fn r#match(&self, segment: &str) -> Option<Slot> {
        let mtc = self.regex_set.matches(segment).into_iter().next()?;

        let regex = &self.regexes[mtc];
        regex
            .find(segment)
            .filter(|m| m.start() == 0 && m.end() == segment.len())
            .map(|_m| self.table[mtc])
    }
}

impl Builder for RegexStrategyBuilder {
    type Strategy = RegexStrategy;
    type Error = regex::Error;

    fn build(self) -> Result<Self::Strategy, Self::Error> {
        Ok(RegexStrategy {
            regex_set: regex::RegexSetBuilder::new(self.patterns.clone())
                .unicode(true)
                .build()?,
            regexes: self
                .patterns
                .into_iter()
                .map(|p| regex::RegexBuilder::new(&p).unicode(true).build())
                .collect::<Result<Vec<_>, _>>()?,
            table: self.table,
        })
    }

    fn add(&mut self, pattern: &str, slot: Slot) -> &mut Self {
        self.patterns.push(pattern.to_string());
        self.table.push(slot);
        self
    }
}
