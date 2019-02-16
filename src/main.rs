mod password;

use std::borrow::Cow;
use std::cmp::{max, min};
use std::fmt::{Debug, Display};
use std::str::FromStr;

use structopt::StructOpt;

use crate::password::Password;

const WORD_LIST: [&str; 16] = [
    "Apple",
    "Banana",
    "Cranberry",
    "Doughnut",
    "Elixer",
    "Fabric",
    "Gregarious",
    "Human",
    "Ignoble",
    "Juniper",
    "Kangaroo",
    "Loup",
    "Machismo",
    "Noteriety",
    "Oragami",
    "Phobos",
];

#[derive(Debug)]
struct NewlineBehaviorParseError;

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
struct Opt {
    /// The number of words in the password
    #[structopt(short = "c", long = "word-count", default_value = "4")]
    word_count: u16,

    /// The maximum length of the password, in bytes. Defaults to unlimited.
    #[structopt(short = "l", long = "max-length")]
    max_length: Option<usize>,

    /// The minimum length of the password, in bytes. Defaults to 24, or MAX_LENGTH,
    /// whichever is lower
    #[structopt(short = "m", long = "min-length")]
    min_length: Option<usize>,

    /// Append a random numeral (0-9) to the password. This is the default.
    ///
    /// Overridden by --no-append-numeral
    #[structopt(short = "n", long = "append-numeral")]
    append_numeral: bool,

    /// Do not append a numeral to the password.
    ///
    /// Overridden by --append-numeral
    #[structopt(
        short = "N",
        long = "no-append-numeral",
        overrides_with = "append_numeral"
    )]
    no_append_numeral: bool,

    /// Append a random special character to the password.
    ///
    /// Overridden by `--no-append-symbol`. See --symbol-set for the default set of special
    /// characters
    #[structopt(short = "%", long = "append-symbol")]
    append_symbol: bool,

    /// Do not append a random special character to the password. This is the default.
    ///
    /// Overridden by --append-symbol and/or --symbol-set.
    #[structopt(long = "no-append-symbol", overrides_with = "append_symbol")]
    no_append_symbol: bool,

    /// The set of symbols to choose from when appending a random symbol.
    ///
    /// Implies --append_symbol. Defaults to !"#$%&'()*+,-./\:;<=>?@[]^_`{|}~
    #[structopt(short = "S", long = "symbol-set", requires = "append_symbol")]
    symbol_set: Option<String>,

    /// The minimum length of each individual word in the password, in bytes. Defaults to 4, or
    /// MAX_WORD, whichever is lower.
    #[structopt(long = "min-word")]
    min_word: Option<usize>,

    /// The maximum length of each individual word in the password, in bytes. Defaults to 8, or
    /// MIN_WORD, whichever is higher.
    #[structopt(long = "max-word")]
    max_word: Option<usize>,

    /// The wordlist from which to select words for the password.
    ///
    /// See --list-wordlist for a list of all available wordlists, and --print-wordlist
    /// for all the words in a given wordlist. This option will also accept "stdin" or "-",
    /// in which case the words will be read (whitespace-separated) from stdin.
    #[structopt(short = "w", long = "wordlist")]
    wordlist: Option<String>,

    /// Print the list of available wordlists to stdout, then exit
    #[structopt(short = "L", long = "list-wordlists")]
    list_wordlists: bool,

    /// Print a complete wordlist to stdout, then exit
    #[structopt(short = "p", long = "print-wordlist")]
    print_wordlist: bool,

    /// The number of passwords to generate when performing entropy estimations.
    ///
    /// Also the number of attempts to create a valid password (for instance, which meets the
    /// length constraints) before giving up.
    #[structopt(short = "s", long = "sample-size", default_value = "100000")]
    sample_size: usize,

    /// Use only the top TOP_WORDS words from the word list (after filtering by size).
    ///
    /// Using a smaller word list will make your password less secure, but possibly easier to
    /// remember. By default, all word lists are sorted by commonality, with more common words
    /// being near the top.
    #[structopt(short = "t", long = "top-words")]
    top_words: Option<usize>,

    /// Print an estimate of the password entropy to stderr
    #[structopt(short = "e", long = "entropy-estimate")]
    entropy_estimate: bool,

    /// Print the password length (in bytes and code points) to stderr
    #[structopt(short = "C", long = "show-count")]
    show_count: bool,

    /// Print entropy estimate calculation details to stderr. Implies --entropy-estimate and
    /// --show-count
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// Trailing newline behavior for the password. If "auto",
    /// a trailing newline will be printed iff stdout is detected to be a tty.
    #[structopt(
        long = "newline",
        default_value = "auto",
        possible_value = "never",
        possible_value = "always",
        possible_value = "auto"
    )]
    newline: NewlineBehavior,
}

#[derive(Debug)]
struct BoundsError {
    min: usize,
    max: usize,
}

#[derive(Debug)]
enum UsageError {
    LengthBoundsError(BoundsError),
    WordBoundsError(BoundsError),
}

impl Opt {
    fn length_bounds(&self) -> Result<(usize, Option<usize>), BoundsError> {
        match (self.min_length, self.max_length) {
            (None, None) => Ok((24, None)),
            (Some(min_length), None) => Ok((min_length, None)),
            (None, Some(max_length)) => Ok((min(24, max_length), Some(max_length))),
            (Some(min_length), Some(max_length)) => {
                if min_length > max_length {
                    Err(BoundsError {
                        min: min_length,
                        max: max_length,
                    })
                } else {
                    Ok((min_length, Some(max_length)))
                }
            }
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

    fn word_length_bounds(&self) -> Result<(usize, usize), BoundsError> {
        match (self.min_word, self.max_word) {
            (None, None) => Ok((4, 8)),
            (Some(min_word), None) => Ok((min_word, max(min_word, 8))),
            (None, Some(max_word)) => Ok((min(4, max_word), max_word)),
            (Some(min_word), Some(max_word)) => {
                if min_word > max_word {
                    Err(BoundsError {
                        min: min_word,
                        max: max_word,
                    })
                } else {
                    Ok((min_word, max_word))
                }
            }
        }
    }

    fn top_words(&self) -> usize {
        self.top_words.unwrap_or(std::usize::MAX)
    }
}

fn main() {
    let opts: Opt = Opt::from_args();

    let (min_word, max_word) = opts.word_length_bounds().unwrap_or_else(|err| bail(err));

    let wordlist: Vec<&str> = WORD_LIST
        .iter()
        .map(|word| *word)
        .filter(|word| word.len() >= min_word && word.len() <= max_word)
        .take(opts.top_words())
        .collect();
}
