use std::env;
use std::process;

use update_nuspec::cli::args::{help_text, parse_args, version};
use update_nuspec::cli::run;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let options = parse_args(&args);

    if options.show_help {
        println!("{}", help_text());
        return;
    }

    if options.show_version {
        println!("update-nuspec {}", version());
        return;
    }

    if let Err(error) = run(&options) {
        eprintln!("{error}");
        process::exit(1);
    }
}
