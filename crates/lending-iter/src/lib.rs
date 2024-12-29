/**
 * @file lib.rs
 * @author Krisna Pranav
 * @brief lib[lending-iter]
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */

mod adapters;

pub trait LendingIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>>;

    fn enumerate(self) -> adapters::Enumerate<Self>
    where
        Self: Sized,
    {
        adapters::Enumerate::new(self)
    }

    fn cloned<'a, T>(self) -> adapters::Cloned<Self>
    where
        Self: Sized,
        for<'b> Self::Item<'b>: std::ops::Deref<Target = T>,
        T: Clone,
    {
        adapters::Cloned::new(self)
    }

    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item<'_>) -> B,
    {
        let mut f = f;
        let mut acc = init;
        let mut iter = self;

        while let Some(item) = iter.next() {
            acc = f(acc, item);
        }

        acc
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.fold(0, |acc, _| acc + 1)
    }

    fn map<B, F>(self, f: F) -> adapters::Map<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Item<'_>) -> B,
    {
        adapters::Map::new(self, f)
    }

    fn filter<F>(self, f: F) -> adapters::Filter<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Item<'_>) -> bool,
    {
        adapters::Filter::new(self, f)
    }

    fn flatten<'a>(self) -> adapters::Flatten<'a, Self>
    where
        Self: Sized,
        Self::Item<'a>: LendingIterator,
    {
        adapters::Flatten::new(self)
    }
}

impl<'a, I> LendingIterator for &'a mut I
where
    I: LendingIterator,
{
    type Item<'b> = I::Item<'b>
    where
        I: 'b,
        'a: 'b;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        (*self).next()
    }
}

pub trait IntoLendingIterator: Sized {
    fn lending(self) -> IntoLending<Self>;
}

pub struct IntoLending<I> {
    iter: I,
}

impl<I> LendingIterator for IntoLending<I>
where
    I: Iterator,
{
    type Item<'a> = I::Item
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        self.iter.next()
    }
}

impl<I> IntoLendingIterator for I
where
    I: Iterator,
{
    fn lending(self) -> IntoLending<Self> {
        IntoLending { iter: self }
    }
}
