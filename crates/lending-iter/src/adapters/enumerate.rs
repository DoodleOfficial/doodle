/**
 * @file enumerate.rs
 * @author Krisna Pranav
 * @brief enumerate
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */

 use crate::LendingIterator;

 pub struct Enumerate<I> {
     iter: I,
     index: usize,
 }
 
 impl<I> Enumerate<I> {
     pub fn new(iter: I) -> Self {
         Self { iter, index: 0 }
     }
 }
 
 impl<I> From<I> for Enumerate<I> {
     fn from(iter: I) -> Self {
         Self::new(iter)
     }
 }
 
 impl<I> LendingIterator for Enumerate<I>
 where
     I: LendingIterator,
 {
     type Item<'a> = (usize, I::Item<'a>)
     where
         I: 'a;
 
     fn next(&mut self) -> Option<Self::Item<'_>> {
         self.iter.next().map(|item| {
             let index = self.index;
             self.index += 1;
             (index, item)
         })
     }
 }