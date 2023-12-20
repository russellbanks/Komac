pub trait IterableExt<T, F>: Iterator {
    fn distinct_or_none(self, selector: F) -> Option<T>
    where
        F: Fn(Self::Item) -> T,
        T: Eq;

    fn single_or_else(self, default: T, selector: F) -> Option<T>
    where
        F: Fn(Self::Item) -> T,
        T: Eq;
}

impl<I, T, F> IterableExt<T, F> for I
where
    I: Iterator,
    F: Fn(I::Item) -> T,
    T: Eq,
{
    fn distinct_or_none(self, selector: F) -> Option<T> {
        let mut values = self.map(selector);

        if let Some(first) = values.next() {
            if values.all(|value| value == first) {
                return Some(first);
            }
        }

        None
    }

    fn single_or_else(self, default: T, selector: F) -> Option<T> {
        let mut values = self.map(selector);

        if let Some(first) = values.next() {
            if values.all(|value| value == first) {
                return None;
            }
        }

        Some(default)
    }
}

#[cfg(test)]
mod tests {
    use crate::iterable_extensions::IterableExt;
    use std::rc::Rc;

    struct RcStr<'a>(Rc<&'a str>);

    #[test]
    fn test_single_or_else_returns_default() {
        let value = Rc::new(RcStr("Value".into()));
        let other_value = Rc::new(RcStr("DifferentValue".into()));
        let actual = vec![value, other_value.clone()]
            .into_iter()
            .single_or_else(other_value.0.clone(), |data| data.0.clone());
        assert_eq!(actual, Some(other_value.0.clone()));
    }

    #[test]
    fn test_single_or_else_returns_none() {
        let value = Rc::new(RcStr("Duplicated value".into()));
        let actual = vec![value.clone(); 3]
            .into_iter()
            .single_or_else(value.0.clone(), |data| data.0.clone());
        assert!(actual.is_none());
    }

    #[test]
    fn test_distinct_or_none_returns_some() {
        let value = Rc::new(RcStr("Duplicated value".into()));
        let actual = vec![value.clone(); 3]
            .into_iter()
            .distinct_or_none(|data| data.0.clone());
        assert_eq!(actual, Some(value.0.clone()));
    }

    #[test]
    fn test_distinct_or_none_returns_none() {
        let actual = vec![RcStr("Value".into()), RcStr("DifferentValue".into())]
            .into_iter()
            .distinct_or_none(|data| data.0.clone());
        assert!(actual.is_none());
    }
}
