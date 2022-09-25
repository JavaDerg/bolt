use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use hashbrown::HashMap;
use itertools::Itertools;
use regex::{RegexSet, RegexSetBuilder};
use std::marker::PhantomData;

pub type LocationMatcher<'a, S, O, C, I, ItE, ItF, ItR> =
    NaiveMatcher<'a, S, O, C, PrefixMatcher<'a, S, O, C, I, ItF, '/'>, I, ItE, ItR>;
pub type DomainMatcher<'a, S, O, C, I, ItE, ItF, ItR> =
    NaiveMatcher<'a, S, O, C, SuffixMatcher<'a, S, O, C, I, ItF, '.'>, I, ItE, ItR>;

pub struct NaiveMatcher<
    'a,
    S,
    O,
    C,
    FM,
    I = &'a str,
    ItE = fn(I) -> &'a str,
    ItR = fn(I) -> &'a str,
> where
    C: Fn(&S) -> O,
    FM: Matcher<S, I, O>,
    ItE: Fn(I) -> &'a str,
    ItR: Fn(I) -> &'a str,
{
    exact: Option<ExactMatcher<'a, S, O, C, I, ItE>>,
    fixed: Option<FM>,
    regex: Option<RegexMatcher<'a, S, O, C, I, ItR>>,

    default: Option<S>,
}

pub trait Matcher<S, I, O>: Sized {
    fn match_(&self, input: I) -> Option<O>;
}

pub struct ExactMatcher<'a, S, O, C: Fn(&S) -> O, I, It: Fn(I) -> &'a str> {
    matches: HashMap<String, S>,
    converter: C,

    input_transformer: It,
    _phantom_input: PhantomData<&'a I>,
}

pub struct PrefixMatcher<'a, S, O, C: Fn(&S) -> O, I, It: Fn(I) -> &'a str, const SEP: char> {
    matcher: AhoCorasick,
    matches: Vec<S>,
    converter: C,

    input_transformer: It,
    _phantom_input: PhantomData<&'a I>,
}

pub struct SuffixMatcher<'a, S, O, C: Fn(&S) -> O, I, It: Fn(I) -> &'a str, const SEP: char> {
    matcher: AhoCorasick,
    matches: Vec<S>,
    converter: C,

    input_transformer: It,
    _phantom_input: PhantomData<&'a I>,
}

pub struct RegexMatcher<'a, S, O, C, I, It: Fn(I) -> &'a str>
where
    C: Fn(&S) -> O,
{
    matcher: RegexSet,
    matches: Vec<S>,
    converter: C,

    input_transformer: It,
    _phantom_input: PhantomData<&'a I>,
}

impl<'a, S, O, C, FM, I, ItE, ItR> NaiveMatcher<'a, S, O, C, FM, I, ItE, ItR>
where
    C: Fn(&S) -> O + Clone,
    FM: Matcher<S, I, O>,
    ItE: Fn(I) -> &'a str,
    ItR: Fn(I) -> &'a str,
{
    pub fn new<FMC: Fn(Vec<(String, S)>, C, ItF) -> FM, ItF: Fn(I) -> &'a str>(
        exact: Vec<(String, S)>,
        fixed: Vec<(String, S)>,
        regex: Vec<(String, S)>,
        default: Option<S>,
        converter: C,
        fixed_matcher_constructor: FMC,
        exact_input_transformer: ItE,
        fixed_input_transformer: ItF,
        regex_input_transformer: ItR,
    ) -> eyre::Result<Self> {
        Ok(Self {
            exact: (!exact.is_empty())
                .then(|| ExactMatcher::build(exact, converter.clone(), exact_input_transformer)),
            fixed: (!fixed.is_empty()).then(|| {
                fixed_matcher_constructor(fixed, converter.clone(), fixed_input_transformer)
            }),
            regex: if !regex.is_empty() {
                Some(RegexMatcher::build(
                    regex,
                    converter.clone(),
                    regex_input_transformer,
                )?)
            } else {
                None
            },
            default,
        })
    }
}

impl<'a, S, O, C, FM, I: Clone, It> Matcher<S, I, O> for NaiveMatcher<'a, S, O, C, FM, I, It>
where
    C: Fn(&S) -> O,
    FM: Matcher<S, I, O>,
    It: Fn(I) -> &'a str,
{
    fn match_(&self, input: I) -> Option<O> {
        self.exact
            .as_ref()
            .and_then(|m| m.match_(input.clone()))
            .or_else(|| self.fixed.as_ref().and_then(|m| m.match_(input.clone())))
            .or_else(|| self.regex.as_ref().and_then(|m| m.match_(input)))
            .or_else(|| self.default.as_ref().map(|m| (self.converter)(m)))
    }
}

impl<'a, S, O, C, I, It> ExactMatcher<'a, S, O, C, I, It>
where
    C: Fn(&S) -> O,
    It: Fn(I) -> &'a str,
{
    pub fn build(entries: Vec<(String, S)>, converter: C, it: It) -> Self {
        Self {
            matches: entries.into_iter().collect(),
            converter,
            input_transformer: it,
            _phantom_input: Default::default(),
        }
    }
}

impl<'a, S, O, C, I, It> Matcher<S, I, O> for ExactMatcher<'a, S, O, C, I, It>
where
    C: Fn(&S) -> O,
    It: Fn(I) -> &'a str,
{
    fn match_(&self, input: I) -> Option<O> {
        let value = self.matches.get((self.input_transformer)(input))?;
        Some((self.converter)(value))
    }
}

impl<'a, S, O, C, I, It, const SEP: char> PrefixMatcher<'a, S, O, C, I, It, SEP>
where
    C: Fn(&S) -> O,
    It: Fn(I) -> &'a str,
{
    pub fn build(entries: Vec<(String, S)>, converter: C, it: It) -> Self {
        let (keys, matches): (Vec<String>, Vec<S>) = entries.into_iter().multiunzip();
        let matcher = AhoCorasickBuilder::new().dfa(true).build(keys);

        Self {
            matcher,
            matches,
            converter,
            input_transformer: it,
            _phantom_input: Default::default(),
        }
    }
}

impl<'a, S, O, C, I, It, const SEP: char> Matcher<S, I, O>
    for PrefixMatcher<'a, S, O, C, I, It, SEP>
where
    C: Fn(&S) -> O,
    It: Fn(I) -> &'a str,
{
    fn match_(&self, input: I) -> Option<O> {
        let input = (self.input_transformer)(input);

        let mut best_case: Option<(usize, usize)> = None;

        for mtch in self.matcher.find_iter(input) {
            if mtch.start() != 0 {
                continue;
            }
            if mtch.end() == input.len() {
                return Some((self.converter)(&self.matches[mtch.pattern()]));
            }
            if input.as_bytes()[mtch.end()] == SEP as u8 {
                if let Some(best_case) = best_case {
                    if best_case.1 > mtch.end() {
                        continue;
                    }
                }
                best_case = Some((mtch.pattern(), mtch.end()));
            }
        }
        let (bc, _) = best_case?;

        Some((self.converter)(&self.matches[bc]))
    }
}

impl<'a, S, O, C, I, It, const SEP: char> SuffixMatcher<'a, S, O, C, I, It, SEP>
where
    C: Fn(&S) -> O,
    It: Fn(I) -> &'a str,
{
    pub fn build(entries: Vec<(String, S)>, converter: C, it: It) -> Self {
        let (keys, matches): (Vec<String>, Vec<S>) = entries.into_iter().multiunzip();
        let matcher = AhoCorasickBuilder::new().dfa(true).build(keys);

        Self {
            matcher,
            matches,
            converter,
            input_transformer: it,
            _phantom_input: Default::default(),
        }
    }
}

impl<'a, S, O, C, I, It, const SEP: char> Matcher<S, I, O>
    for SuffixMatcher<'a, S, O, C, I, It, SEP>
where
    C: Fn(&S) -> O,
    It: Fn(I) -> &'a str,
{
    fn match_(&self, input: I) -> Option<O> {
        let input = (self.input_transformer)(input);

        let mut best_case: Option<(usize, usize)> = None;

        for mtch in self.matcher.find_iter(input) {
            if mtch.end() != input.len() {
                continue;
            }
            if mtch.start() == 0 {
                return Some((self.converter)(&self.matches[mtch.pattern()]));
            }
            if input.as_bytes()[mtch.start() - 1] == SEP as u8 {
                if let Some(best_case) = best_case {
                    if best_case.1 < mtch.start() {
                        continue;
                    }
                }
                best_case = Some((mtch.pattern(), mtch.start()));
            }
        }
        let (bc, _) = best_case?;

        Some((self.converter)(&self.matches[bc]))
    }
}

impl<'a, S, O, C, I, It> RegexMatcher<'a, S, O, C, I, It>
where
    C: Fn(&S) -> O,
    It: Fn(I) -> &'a str,
{
    pub fn build(entries: Vec<(String, S)>, converter: C, it: It) -> Result<Self, regex::Error> {
        let (regex, matches): (Vec<String>, Vec<S>) = entries.into_iter().multiunzip();
        let matcher = RegexSetBuilder::new(regex).unicode(true).build()?;

        Ok(Self {
            matcher,
            matches,
            converter,
            input_transformer: it,
            _phantom_input: Default::default(),
        })
    }
}

impl<'a, S, O, C, I, It> Matcher<S, I, O> for RegexMatcher<'a, S, O, C, I, It>
where
    C: Fn(&S) -> O,
    It: Fn(I) -> &'a str,
{
    fn match_(&self, input: I) -> Option<O> {
        let idx = self
            .matcher
            .matches((self.input_transformer)(input))
            .iter()
            .next()?;
        Some((self.converter)(&self.matches[idx]))
    }
}

pub fn identity<T>(t: T) -> T {
    t
}
