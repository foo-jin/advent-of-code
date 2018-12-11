#!/usr/bin/env bash
set -e

DAY="${1}"
hyperfine "./target/release/day${DAY} < 2018/day${DAY}/input.txt"
