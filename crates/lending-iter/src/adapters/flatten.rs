/**
 * @file flattern.rs
 * @author Krisna Pranav
 * @brief flattern
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */
use crate::LendingIterator;

pub struct Flatten<'a, I>
where
    I: LendingIterator,
    I::Item<'a>: LendingIterator,
    Self: 'a,
{
    iter: I,
    current: Option<I::Item<'a>>,
}

impl<'a, I> Flatten<'a, I>
where
    I: LendingIterator,
    I::Item<'a>: LendingIterator,
{
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            current: None,
        }
    }
}

impl<'a, I> LendingIterator for Flatten<'a, I>
where
    I: LendingIterator,
    I::Item<'a>: LendingIterator,
    Self: 'a,
{
    type Item<'b>
        = <I::Item<'a> as LendingIterator>::Item<'b>
    where
        Self: 'b;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        loop {
            let self_ = unsafe { &mut *(self as *mut Self) };

            if let Some(current) = unsafe { &mut *(&mut self_.current as *mut Option<I::Item<'a>>) }
            {
                if let Some(item) = current.next() {
                    return Some(item);
                }
            }

            self_.current = self_.iter.next();
            self_.current.as_ref()?;
        }
    }
}
