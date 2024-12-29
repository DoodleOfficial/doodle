/**
 * @file map.rs
 * @author Krisna Pranav
 * @brief map
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */
use crate::LendingIterator;

pub struct Map<I, F> {
    iter: I,
    f: F,
}

impl<I, F> Map<I, F> {
    pub fn new(iter: I, f: F) -> Self {
        Self { iter, f }
    }
}

impl<I, F, T> LendingIterator for Map<I, F>
where
    I: LendingIterator,
    F: for<'a> FnMut(I::Item<'a>) -> T,
{
    type Item<'a>
        = T
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        self.iter.next().map(&mut self.f)
    }
}
