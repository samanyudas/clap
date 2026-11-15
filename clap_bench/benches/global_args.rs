//! Used to simulate a large number of global args and subcommands, where a parse only uses one
//! subcommand

#![allow(elided_lifetimes_in_paths)] // needed for divan

use clap::{Arg, ArgAction, ArgMatches, Command};

const SUBCOMMANDS: usize = 100;
const SUBCOMMAND_ARGS: usize = 5;
const GLOBAL_ARGS: usize = 50;

lazy_static::lazy_static! {
    static ref SUBCOMMAND_NAMES: Vec<&'static str> = leaked_names("sub", SUBCOMMANDS);
    static ref SUBCOMMAND_ARG_NAMES: Vec<&'static str> =
        leaked_names("local", SUBCOMMANDS * SUBCOMMAND_ARGS);
    static ref GLOBAL_ARG_NAMES: Vec<&'static str> = leaked_names("global", GLOBAL_ARGS);
}

#[divan::bench]
fn build() -> Command {
    create_app()
}

mod startup {
    use super::{ArgMatches, create_app};

    #[divan::bench]
    fn top_level() -> ArgMatches {
        create_app().get_matches_from(["claptests", "--global-0"])
    }

    #[divan::bench]
    fn sc() -> ArgMatches {
        create_app().get_matches_from(["claptests", "sub-0", "--global-0"])
    }
}

fn create_app() -> Command {
    let mut cmd = Command::new("claptests");
    for name in GLOBAL_ARG_NAMES.iter() {
        cmd = cmd.arg(
            Arg::new(*name)
                .long(*name)
                .help("A global flag")
                .action(ArgAction::SetTrue)
                .global(true),
        );
    }
    for (index, name) in SUBCOMMAND_NAMES.iter().enumerate() {
        let mut sub = Command::new(*name).about("A subcommand");
        let local_names = &SUBCOMMAND_ARG_NAMES[index * SUBCOMMAND_ARGS..][..SUBCOMMAND_ARGS];
        for local in local_names {
            sub = sub.arg(
                Arg::new(*local)
                    .long(*local)
                    .help("A subcommand flag")
                    .action(ArgAction::SetTrue),
            );
        }
        cmd = cmd.subcommand(sub);
    }
    cmd
}

fn leaked_names(prefix: &str, count: usize) -> Vec<&'static str> {
    (0..count)
        .map(|index| &*format!("{prefix}-{index}").leak())
        .collect()
}

fn main() {
    divan::main();
}
