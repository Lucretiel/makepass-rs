use std::fmt::{self, Display, Formatter};
use std::iter;

use rand::{CryptoRng, Rng};
use rand::seq::{SliceRandom, IteratorRandom};

use crate::util::Len;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PasswordRules<'a> {
    pub wordlist: &'a[&'a str],
    pub num_words: usize,
    pub append_numeral: bool,
    pub append_symbol: Option<&'a str>,
}

impl<'a> PasswordRules<'a> {
    fn gen_words<R: CryptoRng + Rng + ?Sized>(&self, rng: &mut R) -> impl Iterator<Item=&'a str> {
        self.wordlist.choose_multiple(rng, self.num_words).cloned()
    }

    fn gen_symbol<R: CryptoRng + Rng + ?Sized>(&self, rng: &mut R) -> Option<char> {
        self.append_symbol.and_then(move |symbol_set| symbol_set.chars().choose(rng))
    }

    fn gen_numeral<R: CryptoRng + Rng + ?Sized>(&self, rng: &mut R) -> Option<u8> {
        if self.append_numeral {
            Some(rng.gen_range(0, 10))
        } else {
            None
        }
    }

    pub fn gen_password<R: CryptoRng + Rng + ?Sized>(&self, rng: &mut R) -> Password<'a> {
        Password {
            words: self.gen_words(rng).collect(),
            numeral: self.gen_numeral(rng),
            symbol: self.gen_symbol(rng),
        }
    }

    pub fn stream_passwords<'s, R: CryptoRng + Rng + ?Sized>(&'s self, rng: &'s mut R) -> impl Iterator<Item=Password<'a>> + 's {
        iter::repeat_with(move || self.gen_password(rng))
    }

    pub fn words_entropy(&self) -> f32 {
        (0..self.num_words)
            .map(|i| self.wordlist.len().checked_sub(i).expect("num_words larger than wordset size"))
            .map(|n| (n as f32).log2())
            .sum()
    }

    pub fn numeral_entropy(&self) -> f32 {
        if self.append_numeral {
            (10f32).log2()
        } else {
            0f32
        }
    }

    pub fn symbol_entropy(&self) -> f32 {
        match self.append_symbol {
            None => 0f32,
            Some(symbol_set) => (symbol_set.chars().count() as f32).log2(),
        }
    }
}

/// Struct type for a password
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Password<'a> {
    words: Vec<&'a str>,
    numeral: Option<u8>,
    symbol: Option<char>,
}

impl<'a> Len for Password<'a> {
    fn len(&self) -> usize {
        // FIXME: ensure that numeral is indeed a single character numeral
        self.words.iter().map(move |word| word.len()).sum::<usize>() +
            self.numeral.map(|_| 1).unwrap_or(0) +
            self.symbol.map(|c| c.len_utf8()).unwrap_or(0)
    }
}

impl<'a> Display for Password<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.words.iter().try_for_each(|word| word.fmt(f))?;

        if let Some(numeral) = self.numeral {
            numeral.fmt(f)?;
        }

        if let Some(symbol) = self.symbol {
            symbol.fmt(f)?;
        }

        Ok(())
    }
}

