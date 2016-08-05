//! Module providing an iterator adaptor which transforms Iterator<(C, S)> into Iterator<(C, S, C)>.
//!
//! Via trait, auto-implements this adaptor for all iterators it knows how to deal with.
//!
//! Humans are good at making lists of alternating data types: we might think of a set of
//! driving directions, say, as an alternating list of waypoints and instructions for how to
//! reach the next waypoint. Computers do less well with such lists; they'd prefer to think
//! of the same sequence as a list of tuples `(initial, instruction, destination)`, where
//! `initial` and `destination` have the same type. However, humans are bad at constructing
//! such lists; we're too likely to accidentally mis-copy one destination to the next initial.
//!
//! This module provides and implements a trait which consumes an Iterator over `(C, S)` and
//! produces an Iterator over `(C, S, C)`, where every initial value matches the final value
//! of the previous item in the sequence. Note that as there's no trailing value,
//! the final S value of the input iterator is ignored and discarded.

/// Transform an Iterator over (C, S) into an Iterator over (C, S, C), duplicating C as necessary.
///
/// See the module-level documentation for more details.
pub trait DuplicateFirst<I, C, S>
    where I: Iterator,
          C: Clone
{
    fn duplicate_first(self) -> DupeFirst<I, C, S>;
}


impl<I, C, S> DuplicateFirst<I, C, S> for I
    where I: Iterator<Item = (C, S)>,
          C: Clone
{
    fn duplicate_first(mut self) -> DupeFirst<I, C, S> {
        DupeFirst {
            previous: self.next(),
            iterator: self,
        }
    }
}

/// An iterator adaptor which transforms `Iterator<(C, S)>` into `Iterator<(C, S, C)>`.
///
/// This `struct` is created by the `duplicate_first()` method on any `Iterator` which
/// maches its signature.
#[derive(Debug)]
pub struct DupeFirst<I, C, S> {
    iterator: I,
    previous: Option<(C, S)>,
}

impl<I, C, S> Iterator for DupeFirst<I, C, S>
    where I: Iterator<Item = (C, S)>,
          C: Clone
{
    type Item = (C, S, C);

    fn next(&mut self) -> Option<Self::Item> {
        let prev = ::std::mem::replace(&mut self.previous, self.iterator.next());
        // Can't use a match statement here even though it would be super clean;
        // it would need to take the form
        //  match (Some((pc, ps)), &Some((ref nc, ref ns))) {...},
        // and you can't (currently?) bind both as move and as ref in the same
        // match statement.
        if let Some((pc, ps)) = prev {
            if let Some((ref nc, _)) = self.previous {
                return Some((pc, ps, nc.clone()));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dupe_first() {
        let items = vec![(1, 2), (3, 4), (5, 6)];
        let expect = vec![(1, 2, 3), (3, 4, 5)];

        assert_eq!(items.into_iter().duplicate_first().collect::<Vec<_>>(),
                   expect);
    }
}
