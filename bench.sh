#!/usr/bin/env bash
set -e

YEAR="${1}"
DAY="${2}"
hyperfine "./target/release/${YEAR}day${DAY} < ${YEAR}/day${DAY}/input.txt"
