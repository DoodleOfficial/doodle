/**
 * @file cloned.rs
 * @author Krisna Pranav
 * @brief cloned
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */
use std::ops::Deref;

use crate::LendingIterator;

pub struct Cloned<I> {
    iter: I,
}

impl<I> Cloned<I> {
    pub fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<I> From<I> for Cloned<I> {
    fn from(iter: I) -> Self {
        Self::new(iter)
    }
}

impl<I, T> Iterator for Cloned<I>
where
    I: LendingIterator,
    for<'a> I::Item<'a>: Deref<Target = T>,
    T: Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|item| item.deref().clone())
    }
}
