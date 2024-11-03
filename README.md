# NAME

Code Signal Exercises

# DESCRIPTION

Code Signal exercises implemented in _Rust_.

Code Signal exercises are mostly meant for _Python_ and _JavaScript_.\
Although those languages have their own place the implementation in _Rust_ is
more interesting since this is missing.

The interesting part about this application are mostly the concepts behind it on
how to:
- implement a `clap` application with **sub-commands** and **sub-command options**
- **serialise** and **deserialise** data structures
- find **main directory** and working directory
- find **files** in the project directory

# EXECUTION

- `cargo run`

The Site can be launched using the `cargo run` Command.
To launch the Site call the `cargo run` Command within the project directory:

            cargo run --

```plain
$ cargo run -- --help
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/code-signal --help`
Code Signal Rust Exercises

Usage: code-signal [OPTIONS] [COMMAND]

Commands:
  apartments  apartment building exercise
  bookshelf   bookshelf listing excercise
  help        Print this message or the help of the given subcommand(s)

Options:
  -v, --verbosity...  Turn debugging information on (use multiple times to increase verbosity)
  -h, --help          Print help
  -V, --version       Print version

$ cargo run -- apartments --help
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/code-signal apartments --help`
apartment building exercise

Usage: code-signal apartments [OPTIONS]

Options:
  -l, --list             lists apartments
  -o, --occupy <OCCUPY>  occupy apartment of given code
  -h, --help             Print help
```

# IMPLEMENTATION

This is the implementation as `clap` **Console Application** with **SubCommands**.

