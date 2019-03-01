use std::fmt::{self, Display, Formatter};

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
    pub fn display(&self) -> DisplayBounds {
        DisplayBounds(self)
    }
}

///
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DisplayBounds<'a>(&'a Bounds);

impl<'a> Display for DisplayBounds<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match (self.0.min, self.0.max) {
            (min, max) if min == max => write!(f, "exactly {}", min),
            (0, std::usize::MAX) | (1, std::usize::MAX) => f.write_str("any number of"),
            (0, max) | (1, max) => write!(f, "up to {}", max),
            (min, std::usize::MAX) => write!(f, "at least {}", min),
            (min, max) => write!(f, "between {} and {}", min, max),
        }
    }
}
