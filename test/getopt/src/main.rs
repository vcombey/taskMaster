#[macro_use]
extern crate serde_derive;
use std::env;

extern crate docopt;
use docopt::Docopt;

#[derive(Deserialize,Debug)]
struct Args {
    arg_file: Option<String>,
    flag_bytes: bool,
    flag_chars: bool,
    flag_lines: bool,
    flag_words: bool,
    flag_help: bool,
    flag_version: bool,
}

const USAGE: &'static str = "
Usage: wc [options] [<file>]

Options:
    -c, --bytes  print the byte counts
    -m, --chars  print the character counts
    -l, --lines  print the newline counts
    -w, --words  print the word counts
    -h, --help  display this help and exit
    -v, --version  output version information and exit
";


fn main() {
    let argv = env::args();

    let args: Args = Docopt::new(USAGE)
                      .and_then(|d| d.argv(argv).deserialize())
                      .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);
}
