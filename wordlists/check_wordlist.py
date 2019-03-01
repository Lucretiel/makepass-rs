#!/usr/bin/env python3

# This script checks a wordlist via stdin and fixes it to stdout. It performs
# the following checks and fixes:
#
# - Any leading or trailing whitespace is removed
# - Empty lines are ignored and kept
# - Lines starting with # are ignored and kept
# - The word is comprised only of alphabetic characters. If this fails, the
#   script throws an error
# - The word is title-cased
#
# Because this script operates on stdin and stdout, it is recommended that you
# use it with a utility like rewrite (https://github.com/Lucretiel/rewrite) or
# sponge (https://joeyh.name/code/moreutils/) to a wordlist file in-place
import sys

words = set()

for line_number, line in enumerate(sys.stdin, 1):
	line = line.rstrip()

	if line == "":
		print(line)
		continue

	if line.startswith("#"):
		print(line)
		continue

	line = line.lstrip()

	# Note: python uses a different definition of "alphabetic" than Rust does. This probably
	# doesn't matter in practice.
	if not line.isalpha():
		print("Non-alphabetic word found on line {}: {}".format(line_number, line), file=sys.stderr)
		sys.exit(1)

	line = line.title()
	if line in words:
		print("Duplicate word found on line {}: {}".format(line_number, line), file=sys.stderr)
		sys.exit(1)

	print(line)
