mod password;
mod wordlists;
mod util;

use crate::util::Len;
use std::iter::FromIterator;
use std::cmp::{max, min};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use std::process::exit;
use std::io::{self, Write};

use structopt::StructOpt;
use rand::rngs::StdRng;
use atty;
use rand::FromEntropy;

use crate::password::PasswordRules;
use crate::wordlists::{WORDLIST_NAMES, WordlistStorage};
use crate::util::Bounds;

#[derive(Debug, Default, Clone, PartialEq, Eq, Copy)]
struct InvalidNewlineBehavior;

impl Display for InvalidNewlineBehavior {
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

impl NewlineBehavior {
    fn should_print_newline(&self) -> bool {
        match self {
            NewlineBehavior::Never => false,
            NewlineBehavior::Always => true,
            NewlineBehavior::Auto => atty::is(atty::Stream::Stdout),
        }
    }
}

impl FromStr for NewlineBehavior {
    type Err = InvalidNewlineBehavior;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("never") {
            Ok(NewlineBehavior::Never)
        } else if s.eq_ignore_ascii_case("always") {
            Ok(NewlineBehavior::Always)
        } else if s.eq_ignore_ascii_case("auto") {
            Ok(NewlineBehavior::Auto)
        } else {
            Err(InvalidNewlineBehavior)
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
struct InvalidWordlistSelection;

impl Display for InvalidWordlistSelection {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("Invalid wordlist selection")
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum WordlistSelection {
    Stdin,
    Named(String),
}

impl FromStr for WordlistSelection {
    type Err = InvalidWordlistSelection;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        s = s.trim();

        if s.eq_ignore_ascii_case("stdin") || s == "-" {
            Ok(WordlistSelection::Stdin)
        } else if s == "" {
            Err(InvalidWordlistSelection)
        } else {
            Ok(WordlistSelection::Named(s.to_lowercase()))
        }
    }
}

/// Help text
#[derive(Debug, Clone, StructOpt)]
#[structopt(
    rename_all = "kebab-case",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp"),
    raw(setting = "structopt::clap::AppSettings::UnifiedHelpMessage")
)]
struct Opt {
    /// The number of words in the password
    #[structopt(short = "c", long, default_value = "4")]
    word_count: u16,

    /// The maximum length of the password, in bytes.
    ///
    /// Defaults to unlimited.
    #[structopt(short = "l", long, value_name = "MAX_LENGTH")]
    max_length: Option<usize>,

    /// The minimum length of the password, in bytes.
    ///
    /// Defaults to 24, or MAX_LENGTH, whichever is lower
    #[structopt(short = "m", long, value_name = "MIN_LENGTH")]
    min_length: Option<usize>,

    /// Append a random numeral (0-9) to the password. This is the default.
    ///
    /// Overridden by --no-append-numeral
    #[structopt(long)]
    append_numeral: bool,

    /// Do not append a numeral to the password.
    ///
    /// Overridden by --append-numeral
    #[structopt(short = "N", long, overrides_with = "append-numeral")]
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
    #[structopt(short, long, requires = "append_symbol", value_name="SYMBOLS")]
    symbol_set: Option<String>,

    /// The minimum length of each individual word in the password, in bytes.
    ///
    /// Defaults to 4, or MAX_WORD, whichever is lower.
    #[structopt(long, value_name="MIN_WORD_LENGTH")]
    min_word: Option<usize>,

    /// The maximum length of each individual word in the password, in bytes.
    ///
    /// Defaults to 8, or MIN_WORD, whichever is higher.
    #[structopt(long, value_name="MAX_WORD_LENGTH")]
    max_word: Option<usize>,

    /// The wordlist from which to select words for the password.
    ///
    /// See --list-wordlist for a list of all available wordlists, and --print-wordlist
    /// for all the words in a given wordlist. This option will also accept "stdin" or "-",
    /// in which case the words will be read (whitespace-separated) from stdin.
    #[structopt(
        short,
        long,
        value_name="WORDLIST",
        default_value="default",
        raw(possible_values = "WORDLIST_NAMES"),
        possible_value="stdin",
        possible_value="-",
    )]
    wordlist: WordlistSelection,

    /// Print the list of available wordlists to stdout, then exit
    #[structopt(
        short = "L",
        long,
        conflicts_with="print_wordlist",
        conflicts_with="print_filtered_wordlist")]
    list_wordlists: bool,

    /// Print a complete wordlist to stdout, then exit
    #[structopt(short, long)]
    print_wordlist: bool,

    /// Print the wordlist after the word-length and top-words filters are applied
    #[structopt(short="P", long)]
    print_filtered_wordlist: bool,

    /// The number of passwords to generate when performing entropy estimations.
    ///
    /// Also the number of attempts to create a valid password (for instance, which meets the
    /// length constraints) before giving up.
    #[structopt(short = "S", long, default_value = "100000")]
    sample_size: usize,

    /// Use only the top TOP_WORDS words from the word list (after filtering by size).
    ///
    /// Using a smaller word list will make your password less secure, but possibly easier to
    /// remember. By default, all word lists are sorted by commonality, with more common words
    /// being near the top.
    #[structopt(short, long, value_name = "TOP_WORDS")]
    top_words: Option<usize>,

    /// Print an estimate of the password entropy to stderr.
    ///
    /// Use --verbose to see details of how this was calculated.
    #[structopt(short, long)]
    entropy_estimate: bool,

    /// Print the password length (in bytes and code points) to stderr.
    #[structopt(short = "C", long)]
    show_count: bool,

    /// Print entropy estimate calculation details to stderr.
    ///
    /// Implies --entropy-estimate and --show-count
    #[structopt(short = "v", long)]
    verbose: bool,

    /// Trailing newline behavior for the password.
    ///
    /// Whether or not to append a newline to the password. If auto, a trailing
    /// newline will be added iff stdout is detected to be a tty.
    #[structopt(
        short = "b",
        long,
        default_value = "auto",
        possible_value = "never",
        possible_value = "always",
        possible_value = "auto",
        value_name = "BEHAVIOR"
    )]
    newline: NewlineBehavior,

    /// Generate a shell completion file to stdout, then exit.
    #[structopt(
        short,
        long,
        raw(possible_values = "&clap::Shell::variants()"),
        value_name = "SHELL"
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
    fn length_bounds(&self) -> Result<Bounds, InvalidBoundsError> {
        match (self.min_length, self.max_length) {
            (None, None) => Ok(Bounds{min: 24, max: std::usize::MAX}),
            (Some(min), None) => Ok(Bounds{min, max: std::usize::MAX}),
            (None, Some(max)) => Ok(Bounds{min: min(24, max), max}),
            (Some(min), Some(max)) if min > max => Err(InvalidBoundsError { min, max }),
            (Some(min), Some(max)) => Ok(Bounds{min, max}),
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
    fn word_length_bounds(&self) -> Result<Bounds, InvalidBoundsError> {
        match (self.min_word, self.max_word) {
            (None, None) => Ok(Bounds{min: 4, max: 8}),
            (Some(min), None) => Ok(Bounds{min, max: max(min, 8)}),
            (None, Some(max)) => Ok(Bounds{min: min(4, max), max}),
            (Some(min), Some(max)) if min > max => Err(InvalidBoundsError { min, max }),
            (Some(min), Some(max)) => Ok(Bounds{min, max}),
        }
    }

    fn top_words(&self) -> usize {
        self.top_words.unwrap_or(std::usize::MAX)
    }
}

fn run(opts: &Opt) -> Result<(), i32> {
    // Early termination cases
    if let Some(shell) = opts.gen_completions {
        Opt::clap().gen_completions_to("makepass", shell, &mut io::stdout().lock());
        return Ok(());
    }

    if opts.list_wordlists {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        return WORDLIST_NAMES.iter().try_for_each(move |name| {
            writeln!(stdout, "{}", name)
        }).map_err(|err| {
            eprintln!("Failed to write to stdout: {}", err);
            1
        });
    }

    let wordlist_storage = match opts.wordlist {
        WordlistSelection::Stdin => {
            eprintln!("Reading wordlist from stdin...");
            WordlistStorage::from_stream(io::stdin().lock()).map_err(|err| {
                eprintln!("Error reading wordlist from stdin: {}", err);
                1
            })?
        },
        WordlistSelection::Named(ref name) => {
            WordlistStorage::from_name(&name).ok_or_else(|| {
                eprintln!("No such wordlist {}", name);
                1
            })?
        }
    };

    let wordlist = wordlist_storage.as_wordlist();

    if opts.print_wordlist {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        return wordlist.iter().try_for_each(move |word| {
            writeln!(stdout, "{}", word)
        }).map_err(|err| {
            eprintln!("Failed to write to stdout: {}", err);
            1
        });
    }

    let word_bounds = opts.word_length_bounds().map_err(|err| {
        eprintln!("Error: minimum word length {} is greater than maximum word length {}", err.min, err.max);
        1
    })?;

    let mut filtered_wordlist = wordlist.iter()
        .filter(move |word| word_bounds.check_len(word).is_ok())
        .take(opts.top_words());

    if opts.print_filtered_wordlist {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();

        return filtered_wordlist.try_for_each(move |word| {
            writeln!(stdout, "{}", word)
        }).map_err(|err| {
            eprintln!("Failed to write to stdout: {}", err);
            1
        });
    }

    let filtered_wordlist = Vec::from_iter(filtered_wordlist);
    let password_rules = PasswordRules{
        wordlist: &filtered_wordlist,
        num_words: opts.word_count as usize,
        append_numeral: opts.should_append_numeral(),
        append_symbol: opts.append_symbol()
    };
    let password_bounds = opts.length_bounds().map_err(|err| {
        eprintln!("Error: minimum password length {} is greater than maximum length {}", err.min, err.max);
        1
    })?;

    let mut rng = StdRng::from_entropy();
    let mut password_stream = password_rules.stream_passwords(&mut rng)
        .take(opts.sample_size)
        .filter(move |password| password_bounds.check_len(password).is_ok());

    let final_password = password_stream.next().ok_or_else(|| {
        eprintln!("Couldn't generate any passwords matchings constraints, after {} attempts", opts.sample_size);
        1
    })?;

    if opts.verbose || opts.entropy_estimate {
        let success_size = 1 + password_stream.count();

        let words_entropy = password_rules.words_entropy();
        let numeral_entropy = password_rules.numeral_entropy();
        let symbol_entropy = password_rules.symbol_entropy();
        let base_entropy = words_entropy + numeral_entropy + symbol_entropy;

        let entropy_adjustment = adjusted_entropy(opts.sample_size, success_size);
        let final_entropy = base_entropy + entropy_adjustment;

        if opts.verbose {
            eprintln!("Generated a password of {word_count} non-repeating words, \
                from a set of {word_set_size} words of {word_length} bytes each: \
                {words_entropy:.2} bits of entropy.",
                word_count = password_rules.num_words,
                word_set_size = filtered_wordlist.len(),
                word_length = word_bounds.display(),
                words_entropy = words_entropy,
            );

            if password_rules.append_numeral {
                eprintln!("A random numeral in the range 0-9 was appended, for an \
                    additional {numeral_entropy:.2} bits of entropy.",
                    numeral_entropy = numeral_entropy,
                );
            }

            if let Some(special_char_set) = password_rules.append_symbol {
                eprintln!("A random special character from the set {special_chars} \
                    was appended, for an additional {symbol_entropy:.2} bits of \
                    entropy",
                    special_chars = special_char_set,
                    symbol_entropy = symbol_entropy
                );
            }

            if success_size != opts.sample_size {
                eprintln!("{sample_size} sample passwords were generated, but only {success_size} \
                    had a length of {password_length} bytes. The entropy estimate was adjusted \
                    accordingly by {adjust_entropy:.2} bits.",
                    sample_size = opts.sample_size,
                    success_size = success_size,
                    password_length = password_bounds.display(),
                    adjust_entropy = entropy_adjustment,
                );
            }
        }

        eprintln!("Estimated total password entropy: {entropy:.2} bits.", entropy=final_entropy);
    }

    if opts.verbose || opts.show_count {
        eprintln!("The password is {} bytes", final_password.len());
    }

    print!("{}", final_password);

    if opts.newline.should_print_newline() {
        println!();
    }

    Ok(())
}

fn adjusted_entropy(sample_size: usize, success_size: usize) -> f32 {
    (success_size as f32).log2() - (sample_size as f32).log2()
}

fn main() {
    let opts = Opt::from_args();
    if let Err(code) = run(&opts) {
        exit(code)
    }
}
