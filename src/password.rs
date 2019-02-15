use std::fmt::{self, Display, Formatter};

/// Struct type for a password
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Password<'a> {
    words: Vec<&'a str>,
    numeral: Option<char>,
    symbol: Option<char>,
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
