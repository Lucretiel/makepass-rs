use structopt::StructOpt;
use clap::{arg_enum, _clap_count_exprs};

arg_enum!{
	#[derive(Debug, Clone, Copy, Eq, PartialEq)]
	enum NewlineBehavior {
		never,
		always,
		auto,
	}
}

/// Help text
#[derive(Debug, Clone, StructOpt)]
struct Opt {
	/// The number of words in the password
	#[structopt(short="c", long="word-count", default_value="4")]
	word_count: u16,

	/// The maximum length of the password, in bytes. Defaults to unlimited.
	#[structopt(short="l", long="max-length")]
	max_length: Option<usize>,

	/// The minimum length of the password, in bytes. Defaults to 24, or MAX_LENGTH,
	/// whichever is lower
	#[structopt(short="m", long="min-length")]
	min_length: Option<usize>,

	/// Append a random numeral (0-9) to the password. This is the default. Overridden by
	/// --no-append-numeral
	#[structopt(short="n", long="append-numeral")]
	append_numeral: bool,

	/// Do not append a numeral to the password. Overridden by --append-numeral
	#[structopt(short="N", long="no-append-numeral", overrides_with="append_numeral")]
	no_append_numeral: bool,

	/// Append a random special character to the password. Overridden by `--no-append-symbol`.
	/// See --symbol-set for the default set of special characters
	#[structopt(short="%", long="append-symbol")]
	append_symbol: bool,

	/// Do not append a random special character to the password. This is the default. Overridden
	/// by --append-symbol and/or --symbol-set.
	#[structopt(long="no-append-symbol", overrides_with="append_symbol")]
	no_append_symbol: bool,

	/// The set of symbols to choose from when appending a random symbol. Defaults to -_()/.,?!;:
	#[structopt(short="S", long="symbol-set", requires="append_symbol")]
	symbol_set: Option<String>,

	/// The minimum length of each individual word in the password, in bytes.
	#[structopt(long="min-word", default_value="4")]
	min_word: u8,

	/// The maximum length of each individual word in the password, in bytes.
	#[structopt(long="max-word", default_value="8")]
	max_word: u8,

	/// The wordlist from which to select words for the password (see --print-wordlist for a list).
	/// This option will also accept "stdin" or "-", in which case the words will be read
	/// (newline-separated) from stdin.
	#[structopt(short="w", long="wordlist")]
	wordlist: Option<String>,

	/// Print the list of available wordlists to stdout, then exit
	#[structopt(short="L", long="list-wordlists")]
	list_wordlists: bool,

	/// Print a complete wordlist to stdout, then exit
	#[structopt(short="p", long="print-wordlist")]
	print_wordlist: bool,

	/// The number of passwords to generate when performing entropy estimations. Also the number
	/// of attempts to create a calid password (for instance, which meets the length constraints)
	/// before giving up.
	#[structopt(short="s", long="sample-size", default_value="100000")]
	sample_size: u32,

	/// Use only the top TOP_WORDS words from the word list (after filtering by size). Using a
	/// smaller word list will make your password less secure, but possibly easier to remember. By
	/// default, all word lists are sorted by commonality, with more common words being near the
	/// top.
	#[structopt(short="t", long="top-words")]
	top_words: Option<u32>,

	/// Print an estimate of the password entropy to stderr
	#[structopt(short="e", long="entropy-estimate")]
	entropy_estimate: bool,

	/// Print the password length (in bytes and code points) to stderr
	#[structopt(short="C", long="show-count")]
	show_count: bool,

	/// Print entropy estimate calculation details to stderr. Implies --entropy-estimate and
	/// --show-count
	#[structopt(short="v", long="verbose")]
	verbose: bool,

	/// Trailing newline behavior for the password. If "auto",
	/// a trailing newline will be printed iff stdout is detected to be a tty.
	#[structopt(long="newline", default_value="auto", raw(possible_values="&NewlineBehavior::variants()"))]
	newline: NewlineBehavior,
}

fn main() {
	let opts = Opt::from_args();
}
