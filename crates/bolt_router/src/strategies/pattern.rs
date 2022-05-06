use std::sync::Arc;
use regex::Regex;
use crate::strategies::{Builder, Slot, Strategy, StrategyMatch};

pub struct RegexStrategy {
    regex_set: regex::RegexSet,
    regexes: Vec<Arc<regex::Regex>>,
    table: Vec<Slot>,
}

#[derive(Default)]
pub struct RegexStrategyBuilder {
    patterns: Vec<String>,
    table: Vec<Slot>,
}

pub struct RegexMatch {
    pattern: Arc<Regex>,
}

impl Strategy for RegexStrategy {
    type Builder = RegexStrategyBuilder;

    fn r#match(&self, segment: &str) -> Option<StrategyMatch> {
        let index = self.regex_set.matches(segment).into_iter().next()?;

        let regex = &self.regexes[index];
        regex
            .find(segment)
            .filter(|m| m.start() == 0 && m.end() == segment.len())
            .map(|m| StrategyMatch {
                slot: self.table[index],
                any: Some(Box::new(RegexMatch {
                    pattern: regex.clone(),
                })),
            })
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
                .map(|p| regex::RegexBuilder::new(&p).unicode(true).build().map(Arc::new))
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn builder() {
        let builder = RegexStrategy::builder();

        assert_eq!(builder.patterns, Vec::<String>::new());
        assert_eq!(builder.table, Vec::<Slot>::new());
    }

    #[test]
    fn builder_add() {
        let mut builder = RegexStrategy::builder();
        builder.add("[a-z]{3}", Slot(0));

        assert_eq!(builder.patterns, vec!["[a-z]{3}".to_string()]);
        assert_eq!(builder.table, vec![Slot(0)]);
    }

    #[test]
    fn build() {
        let strategy = RegexStrategy::builder()
            .add_owned("[a-z]{3}", Slot(0))
            .add_owned("[0-9]{3}", Slot(1))
            .build()
            .unwrap();

        assert_eq!(
            strategy.regex_set.patterns(),
            &["[a-z]{3}".to_string(), "[0-9]{3}".to_string()]
        );
        assert_eq!(strategy.table, vec![Slot(0), Slot(1)]);
    }

    #[test]
    fn r#match() {
        let strategy = RegexStrategy::builder()
            .add_owned("[a-z]{3}", Slot(0))
            .add_owned("[0-9]{3}", Slot(1))
            .build()
            .unwrap();

        assert_eq!(strategy.r#match("abc").map(|m| m.slot), Some(Slot(0)));
        assert_eq!(strategy.r#match("xyz").map(|m| m.slot), Some(Slot(0)));
        assert_eq!(strategy.r#match("123").map(|m| m.slot), Some(Slot(1)));
        assert_eq!(strategy.r#match("789").map(|m| m.slot), Some(Slot(1)));

        assert_eq!(strategy.r#match("ab1").map(|m| m.slot), None);
        assert_eq!(strategy.r#match("abcd").map(|m| m.slot), None);
        assert_eq!(strategy.r#match("1234").map(|m| m.slot), None);
        assert_eq!(strategy.r#match("").map(|m| m.slot), None);
    }
}
