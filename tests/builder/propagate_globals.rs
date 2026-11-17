use clap::{Arg, ArgAction, ArgMatches, Command, error::ErrorKind};

fn get_app() -> Command {
    Command::new("myprog")
        .arg(
            Arg::new("GLOBAL_ARG")
                .long("global-arg")
                .help("Specifies something needed by the subcommands")
                .global(true)
                .action(ArgAction::Set)
                .default_value("default_value"),
        )
        .arg(
            Arg::new("GLOBAL_FLAG")
                .long("global-flag")
                .help("Specifies something needed by the subcommands")
                .global(true)
                .action(ArgAction::Count),
        )
        .subcommand(Command::new("outer").defer(|cmd| cmd.subcommand(Command::new("inner"))))
}

fn get_matches(cmd: Command, argv: &'static str) -> ArgMatches {
    cmd.try_get_matches_from(argv.split(' ').collect::<Vec<_>>())
        .unwrap()
}

fn get_outer_matches(m: &ArgMatches) -> &ArgMatches {
    m.subcommand_matches("outer")
        .expect("could not access outer subcommand")
}

fn get_inner_matches(m: &ArgMatches) -> &ArgMatches {
    get_outer_matches(m)
        .subcommand_matches("inner")
        .expect("could not access inner subcommand")
}

fn top_can_access_arg<T: Into<Option<&'static str>>>(m: &ArgMatches, val: T) -> bool {
    m.get_one::<String>("GLOBAL_ARG").map(|v| v.as_str()) == val.into()
}

fn inner_can_access_arg<T: Into<Option<&'static str>>>(m: &ArgMatches, val: T) -> bool {
    get_inner_matches(m)
        .get_one::<String>("GLOBAL_ARG")
        .map(|v| v.as_str())
        == val.into()
}

fn outer_can_access_arg<T: Into<Option<&'static str>>>(m: &ArgMatches, val: T) -> bool {
    get_outer_matches(m)
        .get_one::<String>("GLOBAL_ARG")
        .map(|v| v.as_str())
        == val.into()
}

fn top_can_access_flag(m: &ArgMatches, present: bool, occurrences: u8) -> bool {
    (m.contains_id("GLOBAL_FLAG") == present)
        && (m.get_one::<u8>("GLOBAL_FLAG").copied() == Some(occurrences))
}

fn inner_can_access_flag(m: &ArgMatches, present: bool, occurrences: u8) -> bool {
    let m = get_inner_matches(m);
    (m.contains_id("GLOBAL_FLAG") == present)
        && (m.get_one::<u8>("GLOBAL_FLAG").copied() == Some(occurrences))
}

fn outer_can_access_flag(m: &ArgMatches, present: bool, occurrences: u8) -> bool {
    let m = get_outer_matches(m);
    (m.contains_id("GLOBAL_FLAG") == present)
        && (m.get_one::<u8>("GLOBAL_FLAG").copied() == Some(occurrences))
}

#[test]
fn global_arg_used_top_level() {
    let m = get_matches(get_app(), "myprog --global-arg=some_value outer inner");

    assert!(top_can_access_arg(&m, "some_value"));
    assert!(inner_can_access_arg(&m, "some_value"));
    assert!(outer_can_access_arg(&m, "some_value"));
}

#[test]
fn global_arg_used_outer() {
    let m = get_matches(get_app(), "myprog outer --global-arg=some_value inner");

    assert!(top_can_access_arg(&m, "some_value"));
    assert!(inner_can_access_arg(&m, "some_value"));
    assert!(outer_can_access_arg(&m, "some_value"));
}

#[test]
fn global_arg_used_inner() {
    let m = get_matches(get_app(), "myprog outer inner --global-arg=some_value");

    assert!(top_can_access_arg(&m, "some_value"));
    assert!(inner_can_access_arg(&m, "some_value"));
    assert!(outer_can_access_arg(&m, "some_value"));
}

#[test]
fn global_arg_default_value() {
    let m = get_matches(get_app(), "myprog outer inner");

    assert!(top_can_access_arg(&m, "default_value"));
    assert!(inner_can_access_arg(&m, "default_value"));
    assert!(outer_can_access_arg(&m, "default_value"));
}

#[test]
fn global_flag_used_top_level() {
    let m = get_matches(get_app(), "myprog --global-flag outer inner");

    assert!(top_can_access_flag(&m, true, 1));
    assert!(inner_can_access_flag(&m, true, 1));
    assert!(outer_can_access_flag(&m, true, 1));
}

#[test]
fn global_flag_used_outer() {
    let m = get_matches(get_app(), "myprog outer --global-flag inner");

    assert!(top_can_access_flag(&m, true, 1));
    assert!(inner_can_access_flag(&m, true, 1));
    assert!(outer_can_access_flag(&m, true, 1));
}

#[test]
fn global_flag_used_inner() {
    let m = get_matches(get_app(), "myprog outer inner --global-flag");

    assert!(top_can_access_flag(&m, true, 1));
    assert!(inner_can_access_flag(&m, true, 1));
    assert!(outer_can_access_flag(&m, true, 1));
}

#[test]
fn global_flag_2x_used_top_level() {
    let m = get_matches(get_app(), "myprog --global-flag --global-flag outer inner");

    assert!(top_can_access_flag(&m, true, 2));
    assert!(inner_can_access_flag(&m, true, 2));
    assert!(outer_can_access_flag(&m, true, 2));
}

#[test]
fn global_flag_2x_used_inner() {
    let m = get_matches(get_app(), "myprog outer inner --global-flag --global-flag");

    assert!(top_can_access_flag(&m, true, 2));
    assert!(inner_can_access_flag(&m, true, 2));
    assert!(outer_can_access_flag(&m, true, 2));
}

fn get_app_with_unused_subcommand() -> Command {
    get_app()
        .version("1.0")
        .propagate_version(true)
        .subcommand(Command::new("unused"))
}

fn has_global_arg(cmd: &Command) -> bool {
    cmd.get_arguments().any(|a| a.get_id() == "GLOBAL_ARG")
}

#[test]
fn unused_subcommand_after_parse() {
    let mut cmd = get_app_with_unused_subcommand();
    cmd.try_get_matches_from_mut(["myprog", "outer"]).unwrap();

    let outer = cmd.find_subcommand("outer").unwrap();
    assert!(has_global_arg(outer));
    assert_eq!(outer.get_version(), Some("1.0"));

    let unused = cmd.find_subcommand("unused").unwrap();
    assert!(!has_global_arg(unused));
    assert_eq!(unused.get_version(), None);
}

#[test]
fn unused_subcommand_after_build() {
    let mut cmd = get_app_with_unused_subcommand();
    cmd.try_get_matches_from_mut(["myprog", "outer"]).unwrap();
    cmd.build();

    let unused = cmd.find_subcommand("unused").unwrap();
    assert!(has_global_arg(unused));
    assert_eq!(unused.get_version(), Some("1.0"));
}

#[test]
fn global_positional_with_subcommand_positional() {
    let m = Command::new("myprog")
        .arg(Arg::new("global").global(true))
        .subcommand(Command::new("outer").arg(Arg::new("local")))
        .try_get_matches_from(["myprog", "outer", "first", "second"])
        .unwrap();

    let m = m.subcommand_matches("outer").unwrap();
    assert_eq!(
        m.get_one::<String>("local").map(String::as_str),
        Some("first")
    );
    assert_eq!(
        m.get_one::<String>("global").map(String::as_str),
        Some("second")
    );
}

#[test]
fn global_arg_ignores_parent_local_settings() {
    let res = Command::new("myprog")
        .allow_negative_numbers(true)
        .arg(Arg::new("num").long("num").global(true))
        .subcommand(Command::new("outer"))
        .try_get_matches_from(["myprog", "outer", "--num", "-1"]);

    assert!(res.is_err());
}

#[test]
fn deferred_global_setting_after_build() {
    let mut cmd = Command::new("myprog").infer_subcommands(true).subcommand(
        Command::new("outer")
            .defer(|cmd| cmd.infer_subcommands(false))
            .subcommand(Command::new("inner")),
    );
    cmd.build();

    let res = cmd.try_get_matches_from_mut(["myprog", "outer", "inn"]);
    assert!(res.is_err());
}

#[test]
fn unused_subcommand_help_after_parse() {
    let mut cmd = get_app_with_unused_subcommand();
    cmd.try_get_matches_from_mut(["myprog", "outer"]).unwrap();

    let help = cmd
        .find_subcommand_mut("unused")
        .unwrap()
        .render_help()
        .to_string();
    assert!(help.contains("--global-arg"), "{help}");

    let m = cmd
        .try_get_matches_from_mut(["myprog", "unused", "--global-arg", "some_value"])
        .unwrap();
    let m = m.subcommand_matches("unused").unwrap();
    assert_eq!(
        m.get_one::<String>("GLOBAL_ARG").map(String::as_str),
        Some("some_value")
    );
}

#[test]
fn unused_subcommand_change_after_lookup() {
    let mut cmd = get_app_with_unused_subcommand().disable_help_flag(true);
    cmd.try_get_matches_from_mut(["myprog", "outer"]).unwrap();

    let unused = cmd.find_subcommand_mut("unused").unwrap();
    *unused = std::mem::take(unused).disable_help_flag(false);

    let res = cmd
        .find_subcommand_mut("unused")
        .unwrap()
        .try_get_matches_from_mut(["unused", "--help"]);
    assert_eq!(res.unwrap_err().kind(), ErrorKind::DisplayHelp);
}

#[test]
fn mut_unused_subcommand_after_parse() {
    let mut cmd = get_app_with_unused_subcommand();
    cmd.try_get_matches_from_mut(["myprog", "outer"]).unwrap();

    let mut cmd = cmd.mut_subcommand("unused", |sc| {
        sc.mut_arg("GLOBAL_ARG", |a| a.long("renamed-global-arg"))
    });
    let m = cmd
        .try_get_matches_from_mut(["myprog", "unused", "--renamed-global-arg", "some_value"])
        .unwrap();
    let m = m.subcommand_matches("unused").unwrap();
    assert_eq!(
        m.get_one::<String>("GLOBAL_ARG").map(String::as_str),
        Some("some_value")
    );
}

#[test]
fn unused_subcommand_cloned_into_new_parent() {
    let mut cmd = get_app_with_unused_subcommand();
    cmd.try_get_matches_from_mut(["myprog", "outer"]).unwrap();
    let unused = cmd.find_subcommand_mut("unused").unwrap().clone();

    let m = Command::new("other")
        .arg(
            Arg::new("OTHER_GLOBAL")
                .long("other-global")
                .global(true)
                .action(ArgAction::SetTrue),
        )
        .subcommand(unused)
        .try_get_matches_from(["other", "unused", "--other-global"])
        .unwrap();
    let m = m.subcommand_matches("unused").unwrap();
    assert!(m.get_flag("OTHER_GLOBAL"));
}
