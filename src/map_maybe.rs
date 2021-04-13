pub trait MapMaybeExt<T, O, F: Fn(T) -> Option<O>> {
    fn map_maybe(self, f: F) -> Option<O>;
}

impl<T, O, F: Fn(T) -> Option<O>> MapMaybeExt<T, O, F> for Option<T> {
    fn map_maybe(self, f: F) -> Option<O> {
        match self {
            Some(t) => f(t),
            None => None,
        }
    }
}
