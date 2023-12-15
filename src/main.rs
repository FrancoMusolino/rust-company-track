use rust_company_track::run;
use std::process;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error running the program: {err}");
        process::exit(1);
    }
}
