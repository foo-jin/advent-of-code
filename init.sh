#!/usr/bin/env bash
set -e

YEAR="${1}"
DAY="${2}"
SOURCE="template"
DEST="${YEAR}/day${DAY}"

mkdir "${DEST}"
cp -r "${SOURCE}/"* "${DEST}/"
sed -i "s/template/${YEAR}day${DAY}/g" "${DEST}/Cargo.toml"
sed -i "/members = \[/a\    '${DEST}'," Cargo.toml

cd "${DEST}"
aoc fetch -d "${DAY}" > input.txt
