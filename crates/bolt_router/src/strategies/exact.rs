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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn builder() {
        let builder = ExactStrategy::builder();

        assert_eq!(builder.table.len(), 0);
        assert_eq!(builder.table.into_iter().collect::<Vec<_>>(), vec![]);
    }

    #[test]
    fn builder_add() {
        let mut strategy = ExactStrategy::builder();
        strategy.add("hi.com", Slot(1));

        assert_eq!(strategy.table.len(), 1);
        assert_eq!(
            strategy.table.into_iter().collect::<Vec<_>>(),
            vec![("hi.com".to_string(), Slot(1))]
        );
    }

    #[test]
    fn build() {
        let strategy = ExactStrategy::builder()
            .add_owned("hi.com", Slot(1))
            .add_owned("www.hi.com", Slot(2))
            .build()
            .unwrap();

        assert_eq!(strategy.table.len(), 2);

        let mut vec = strategy.table.into_iter().collect::<Vec<_>>();
        // Vec must be presorted by slot as that's the order we check it by
        vec.sort_by(|x, y| x.1 .0.cmp(&y.1 .0));

        assert_eq!(
            vec,
            vec![
                ("hi.com".to_string(), Slot(1)),
                ("www.hi.com".to_string(), Slot(2)),
            ]
        );
    }

    #[test]
    fn r#match() {
        let strategy = ExactStrategy::builder()
            .add_owned("hi.com", Slot(1))
            .add_owned("www.hi.com", Slot(2))
            .build()
            .unwrap();

        assert_eq!(strategy.r#match("hi.com"), Some(Slot(1)));
        assert_eq!(strategy.r#match("www.hi.com"), Some(Slot(2)));
        assert_eq!(strategy.r#match("www.hi.com.com"), None);
        assert_eq!(strategy.r#match("www.www.hi.com"), None);
    }
}
