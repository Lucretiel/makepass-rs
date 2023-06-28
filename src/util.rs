use std::fmt::Display;

use lazy_format::lazy_format;

pub trait Len {
    fn len(&self) -> usize;
}

impl Len for str {
    fn len(&self) -> usize {
        str::len(self)
    }
}

impl<'a, T: Len + ?Sized> Len for &'a T {
    fn len(&self) -> usize {
        T::len(self)
    }
}

// This struct encompasses an inclusive [min, max] range and is used for checking
// the lengths of things.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Bounds {
    pub min: usize,
    pub max: usize,
}

#[derive(Debug, Clone)]
pub enum BoundsError {
    TooHigh(usize),
    TooLow(usize),
}

impl Bounds {
    pub fn check(&self, value: usize) -> Result<usize, BoundsError> {
        if value < self.min {
            Err(BoundsError::TooLow(self.min))
        } else if value > self.max {
            Err(BoundsError::TooHigh(self.max))
        } else {
            Ok(value)
        }
    }

    pub fn check_len<T: Len>(&self, thing: T) -> Result<T, BoundsError> {
        self.check(thing.len()).map(move |_| thing)
    }

    /// Write these bounds to a stream, in a format satisying "a length of {} bytes"
    pub fn display(&self) -> impl Display {
        let min = self.min;
        let max = self.max;

        lazy_format!(match ((min, max)) {
            (min, max) if min == max => "exactly {min}",
            (0 | 1, std::usize::MAX) => "any number of",
            (0 | 1, max) => "up to {max}",
            (min, std::usize::MAX) => "at least {min}",
            (min, max) => "between {min} and {max}",
        })
    }
}
