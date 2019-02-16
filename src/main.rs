mod password;
mod wordlists;

use std::borrow::Cow;
use std::cmp::{max, min};
use std::fmt::{self, Debug, Display, Formatter};
use std::str::FromStr;

use clap;
use structopt::StructOpt;

use crate::password::Password;
use crate::wordlists::{WORD_LISTS, WORD_LIST_NAMES};

#[derive(Debug)]
struct NewlineBehaviorParseError;

impl Display for NewlineBehaviorParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("Invalid pattern for newline behavior")
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum NewlineBehavior {
    Never,
    Always,
    Auto,
}

impl FromStr for NewlineBehavior {
    type Err = NewlineBehaviorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("never") {
            Ok(NewlineBehavior::Never)
        } else if s.eq_ignore_ascii_case("always") {
            Ok(NewlineBehavior::Always)
        } else if s.eq_ignore_ascii_case("auto") {
            Ok(NewlineBehavior::Auto)
        } else {
            Err(NewlineBehaviorParseError)
        }
    }
}

/// Help text
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct Opt {
    /// The number of words in the password
    #[structopt(short = "c", long, default_value = "4")]
    word_count: u16,

    /// The maximum length of the password, in bytes. Defaults to unlimited.
    #[structopt(short = "l", long)]
    max_length: Option<usize>,

    /// The minimum length of the password, in bytes. Defaults to 24, or MAX_LENGTH,
    /// whichever is lower
    #[structopt(short = "m", long)]
    min_length: Option<usize>,

    /// Append a random numeral (0-9) to the password. This is the default.
    ///
    /// Overridden by --no-append-numeral
    #[structopt(long)]
    append_numeral: bool,

    /// Do not append a numeral to the password.
    ///
    /// Overridden by --append-numeral
    #[structopt(short = "N", long, overrides_with = "append_numeral")]
    no_append_numeral: bool,

    /// Append a random special character to the password.
    ///
    /// Overridden by `--no-append-symbol`. See --symbol-set for the default set of special
    /// characters
    #[structopt(short = "%", long)]
    append_symbol: bool,

    /// Do not append a random special character to the password. This is the default.
    ///
    /// Overridden by --append-symbol and/or --symbol-set.
    #[structopt(long, overrides_with = "append_symbol")]
    no_append_symbol: bool,

    /// The set of symbols to choose from when appending a random symbol.
    ///
    /// Implies --append_symbol. Defaults to !"#$%&'()*+,-./\:;<=>?@[]^_`{|}~. If invoking
    /// from the shell, make sure to properly escape your symbols.
    #[structopt(short, long, requires = "append_symbol")]
    symbol_set: Option<String>,

    /// The minimum length of each individual word in the password, in bytes. Defaults to 4, or
    /// MAX_WORD, whichever is lower.
    #[structopt(long)]
    min_word: Option<usize>,

    /// The maximum length of each individual word in the password, in bytes. Defaults to 8, or
    /// MIN_WORD, whichever is higher.
    #[structopt(long)]
    max_word: Option<usize>,

    /// The wordlist from which to select words for the password.
    ///
    /// See --list-wordlist for a list of all available wordlists, and --print-wordlist
    /// for all the words in a given wordlist. This option will also accept "stdin" or "-",
    /// in which case the words will be read (whitespace-separated) from stdin.
    #[structopt(short, long, raw(possible_values = "&WORD_LIST_NAMES"))]
    wordlist: Option<String>,

    /// Print the list of available wordlists to stdout, then exit
    #[structopt(short = "L", long)]
    list_wordlists: bool,

    /// Print a complete wordlist to stdout, then exit
    #[structopt(short = "p", long)]
    print_wordlist: bool,

    /// The number of passwords to generate when performing entropy estimations.
    ///
    /// Also the number of attempts to create a valid password (for instance, which meets the
    /// length constraints) before giving up.
    #[structopt(short="S", long, default_value = "100000")]
    sample_size: usize,

    /// Use only the top TOP_WORDS words from the word list (after filtering by size).
    ///
    /// Using a smaller word list will make your password less secure, but possibly easier to
    /// remember. By default, all word lists are sorted by commonality, with more common words
    /// being near the top.
    #[structopt(short, long, value_name = "TOP_WORDS")]
    top_words: Option<usize>,

    /// Print an estimate of the password entropy to stderr
    #[structopt(short, long)]
    entropy_estimate: bool,

    /// Print the password length (in bytes and code points) to stderr
    #[structopt(short = "C", long)]
    show_count: bool,

    /// Print entropy estimate calculation details to stderr. Implies --entropy-estimate and
    /// --show-count
    #[structopt(short = "v", long)]
    verbose: bool,

    /// Trailing newline behavior for the password. If "auto",
    /// a trailing newline will be printed iff stdout is detected to be a tty.
    #[structopt(
        short = "b",
        long,
        default_value = "auto",
        possible_value = "never",
        possible_value = "always",
        possible_value = "auto",
        value_name = "behavior"
    )]
    newline: NewlineBehavior,

    /// Generate a shell completion file to stdout, then exit.
    #[structopt(
        short,
        long,
        raw(possible_values = "&clap::Shell::variants()"),
        value_name = "shell"
    )]
    gen_completions: Option<clap::Shell>,
}

// InvalidBoundsError is an error indicating that a set of bounds couldn't be
// calculated, because the min was greated than the max
#[derive(Debug)]
struct InvalidBoundsError {
    min: usize,
    max: usize,
}

impl Opt {
    // Get the user's requests length bounds for the whole password
    fn length_bounds(&self) -> Result<(usize, Option<usize>), InvalidBoundsError> {
        match (self.min_length, self.max_length) {
            (None, None) => Ok((24, None)),
            (Some(min_length), None) => Ok((min_length, None)),
            (None, Some(max_length)) => Ok((min(24, max_length), Some(max_length))),
            (Some(min), Some(max)) if min > max => Err(InvalidBoundsError { min, max }),
            (Some(min), Some(max)) => Ok((min, Some(max))),
        }
    }

    fn should_append_numeral(&self) -> bool {
        // Explaination of logic: if append_numeral is given, it'll override
        // no_append_numeral. If neither are given, the default is true.
        !self.no_append_numeral
    }

    /// If a symbol should be appended, return the set of symbols to choose from.
    fn append_symbol(&self) -> Option<&str> {
        if self.append_symbol {
            Some("!\"#$%&'()*+,-./\\:;<=>?@[]^_`{|}~")
        } else if let Some(ref user_symbols) = self.symbol_set {
            Some(user_symbols.as_str())
        } else {
            None
        }
    }

    // Get the user's requested length bounds for each word
    fn word_length_bounds(&self) -> Result<(usize, usize), InvalidBoundsError> {
        match (self.min_word, self.max_word) {
            (None, None) => Ok((4, 8)),
            (Some(min_word), None) => Ok((min_word, max(min_word, 8))),
            (None, Some(max_word)) => Ok((min(4, max_word), max_word)),
            (Some(min), Some(max)) if min > max => Err(InvalidBoundsError { min, max }),
            (Some(min), Some(max)) => Ok((min, max)),
        }
    }

    fn top_words(&self) -> usize {
        self.top_words.unwrap_or(std::usize::MAX)
    }
}

fn main() {
    let x: Password;
    let opts = Opt::from_args();
}
