#!/usr/bin/env bash
set -e

YEAR="${1}"
DAY="${2}"
LEVEL="${3}"
# cd "${YEAR}/day${DAY}"
cat input.txt | cargo run --release | aoc submit -d "${DAY}" -l "${LEVEL}"
