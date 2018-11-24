#!/usr/bin/env bash
set -e

YEAR="${1}"
DAY="${2}"
DAY_NO_ZEROS="$(echo $DAY | sed 's/^0*//')"
SOURCE="template"
DEST="${YEAR}/day${DAY}"

mkdir "${DEST}"
cp -r "${SOURCE}/"* "${DEST}/"
sed -i "s/template/day${DAY}/g" "${DEST}/Cargo.toml"
sed -i "/members = \[/a\    '${YEAR}\/${DAY}'" Cargo.toml

cd "${DEST}"
aoc fetch -d "${DAY}" > input.txt
