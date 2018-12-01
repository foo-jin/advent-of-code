# Advent of Code
This repo contains my solutions for the Advent of Code problems. Soon I will start solving AoC 2018 in this repository. I have another AoC repository [adventofcode](https://github.com/foo-jin/adventofcode) containing the solutions to all of the 2017 puzzles and some of the 2016 puzzles. I made this repo instead of continuing the old one because I wasn't quite satisfied with the monolithic CLI setup.

## Structure
The root folder is a cargo workspace, with the template project and folders for
different years of AoC. In each year folder, the daily solutions are in separate sub-projects.

## Setup
`init.sh` is used to copy the template to the appropriate location and to
download the input file. `run.sh` is then used to run the program and submit the solution. These scripts use
[aoc-tools](https://github.com/foo-jin/aoc-tools), a small toolset I wrote to fetch input, submit
solutions, and view leaderboards.
