# mpk-mini-ctl

Rust command line tool for the mpk mini.

[![Continuous integration](https://github.com/eldad/mpk-mini-ctl/actions/workflows/rust.yml/badge.svg)](https://github.com/eldad/mpk-mini-ctl/actions/workflows/rust.yml)

## Usage

```
mpk-mini-ctl 0.1.1
Eldad Zack <eldad@fogrefinery.com>
AKAI MPK Mini mkII Control Tool

Usage: mpk-mini-ctl [OPTIONS] <COMMAND>

Commands:
  snoop               Snoop MIDI messages
  passthrough         Passthrough (while snooping) MIDI messages
  show-bank           Show bank settings
  show-ram            Show current active settings (RAM)
  read-file           Read yaml bank descriptor from file and display it
  dump-bank-settings  Dump bank settings as yaml
  dump-ram-settings   Dump current active settings (RAM) as yaml
  load-bank           Read yaml bank descriptor from file and apply it on a bank
  load-ram            Read yaml bank descriptor from file and apply it to active settings (RAM)
  autocompletion      Install local bash auto-completion
  help                Print this message or the help of the given subcommand(s)

Options:
      --debug    Prints debugging information
  -h, --help     Print help
  -V, --version  Print version
```
