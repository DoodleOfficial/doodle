/**
 * @file filter.rs
 * @author Krisna Pranav
 * @brief filter
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */
use crate::LendingIterator;

pub struct Filter<I, P> {
    iter: I,
    predicate: P,
}

impl<I, P> Filter<I, P> {
    pub fn new(iter: I, predicate: P) -> Self {
        Self { iter, predicate }
    }
}

impl<I, P> LendingIterator for Filter<I, P>
where
    I: LendingIterator,
    for<'a> P: FnMut(&I::Item<'a>) -> bool,
{
    type Item<'a>
        = I::Item<'a>
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        loop {
            let self_ = unsafe { &mut *(self as *mut Self) };
            if let Some(item) = self_.iter.next() {
                if (self_.predicate)(&item) {
                    return Some(item);
                }
            } else {
                return None;
            }
        }
    }
}
