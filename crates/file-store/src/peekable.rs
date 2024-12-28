/**
 * @file peekable.rs
 * @author Krisna Pranav
 * @brief peekable
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */

pub struct Peekable<I>
where
    I: Iterator,
{
    iter: I,
    peeked: Option<I::Item>,
}

impl<I> Peekable<I>
where
    I: Iterator,
{
    pub fn new(iter: I) -> Self {
        let mut iter = iter;
        let peeked = iter.next();
        Self { iter, peeked }
    }

    pub fn peek(&self) -> Option<&I::Item> {
        self.peeked.as_ref()
    }
}

impl<I> Iterator for Peekable<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let peeked = self.peeked.take();
        if peeked.is_some() {
            self.peeked = self.iter.next();
        }

        peeked
    }
}

impl<I, T> Ord for Peekable<I>
where
    I: Iterator<Item = T>,
    T: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.peek(), other.peek()) {
            (Some(a), Some(b)) => a.cmp(b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    }
}

impl<I, T> PartialOrd for Peekable<I>
where
    I: Iterator<Item = T>,
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self.peek(), other.peek()) {
            (Some(a), Some(b)) => a.partial_cmp(b),
            (Some(_), None) => Some(std::cmp::Ordering::Less),
            (None, Some(_)) => Some(std::cmp::Ordering::Greater),
            (None, None) => Some(std::cmp::Ordering::Equal),
        }
    }
}

impl<I, T> PartialEq for Peekable<I>
where
    I: Iterator<Item = T>,
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self.peek(), other.peek()) {
            (Some(a), Some(b)) => a == b,
            (None, None) => true,
            _ => false,
        }
    }
}

impl<I, T> Eq for Peekable<I>
where
    I: Iterator<Item = T>,
    T: Eq,
{
}

impl<I, T> std::fmt::Debug for Peekable<I>
where
    I: Iterator<Item = T>,
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Peekable")
            .field("peeked", &self.peeked)
            .finish()
    }
}
