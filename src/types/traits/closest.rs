pub trait Closest {
    fn closest<'iter, I, T>(&self, candidates: I) -> Option<&'iter Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<&'iter Self>,
    {
        candidates
            .into_iter()
            .map(T::into)
            .min_by_key(|candidate| self.distance_key(candidate))
    }

    fn distance_key(&self, other: &Self) -> impl Ord;
}
