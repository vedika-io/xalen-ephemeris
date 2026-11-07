//! `xalen`: the XALEN Ephemeris on the command line.
//!
//! Every command is a thin layer over the library crates: it parses the
//! arguments, calls the library and prints the result as a table or as JSON.
//! There is no astronomy in this crate.
//!
//! Exit codes: `0` success, `1` the computation failed, `2` bad input.

mod aspects;
mod chart;
mod cli;
mod common;
mod convert;
mod dasha;
mod eclipses;
mod error;
mod input;
mod output;
mod panchang;
mod positions;
mod rise_set;
mod star;

use std::io::ErrorKind;
use std::process::ExitCode;

use clap::FromArgMatches;
use clap::error::ErrorKind as ClapKind;

use cli::{Cli, Command};
use error::Result;
use output::Report;

/// Run a command, returning its report and whether JSON was asked for.
fn run(command: &Command) -> (Result<Report>, bool) {
    match command {
        Command::Positions(a) => (positions::run(a), a.out.json),
        Command::Chart(a) => (chart::run(a), a.out.json),
        Command::Panchang(a) => (panchang::run(a), a.out.json),
        Command::Dasha(a) => (dasha::run(a), a.out.json),
        Command::Star(a) => (star::run(a), a.out.json),
        Command::Eclipses(a) => (eclipses::run(a), a.out.json),
        Command::RiseSet(a) => (rise_set::run(a), a.out.json),
        Command::Aspects(a) => (aspects::run(a), a.out.json),
        Command::Convert(a) => (convert::run(a), a.out.json),
    }
}

/// Turn a `clap` error into a one-line message and an exit code.
fn clap_failure(error: clap::Error) -> ExitCode {
    if !error.use_stderr() {
        // --help and --version: printed on stdout, success.
        let _ = error.print();
        return ExitCode::SUCCESS;
    }
    if error.kind() == ClapKind::DisplayHelpOnMissingArgumentOrSubcommand {
        let _ = error.print();
        return ExitCode::from(2);
    }
    let text = error.render().to_string();
    let first = text
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("error: invalid arguments");
    eprintln!("{first} (see --help)");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let matches = match cli::build_command().try_get_matches() {
        Ok(matches) => matches,
        Err(error) => return clap_failure(error),
    };
    let parsed = match Cli::from_arg_matches(&matches) {
        Ok(parsed) => parsed,
        Err(error) => return clap_failure(error),
    };
    let (result, json) = run(&parsed.command);
    match result {
        Ok(report) => match output::emit(&report, json) {
            Ok(()) => ExitCode::SUCCESS,
            // `xalen ... | head` closes the pipe early; that is not a failure.
            Err(error) if error.kind() == ErrorKind::BrokenPipe => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("error: cannot write output: {error}");
                ExitCode::from(1)
            }
        },
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}
