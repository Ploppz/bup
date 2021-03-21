use iced::{Element, Row, Column};

pub trait ColumnExt<'a, M> {
    fn push_iter<I, E>(self, iter: I) -> Self
    where
        I: Iterator<Item = E>,
        E: Into<Element<'a, M>>;
}
impl<'a, M> ColumnExt<'a, M> for Column<'a, M> {
    fn push_iter<I, E>(mut self, iter: I) -> Self
    where
        I: Iterator<Item = E>,
        E: Into<Element<'a, M>>,
    {
        for item in iter {
            self = self.push(item);
        }
        self
    }
}

pub trait RowExt<'a, M> {
    fn push_iter<I, E>(self, iter: I) -> Self
    where
        I: Iterator<Item = E>,
        E: Into<Element<'a, M>>;
}
impl<'a, M> RowExt<'a, M> for Row<'a, M> {
    fn push_iter<I, E>(mut self, iter: I) -> Self
    where
        I: Iterator<Item = E>,
        E: Into<Element<'a, M>>,
    {
        for item in iter {
            self = self.push(item);
        }
        self
    }
}
