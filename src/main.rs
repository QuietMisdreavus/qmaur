// qmaur - QuietMisdreavus AUR tool
// Copyright (C) 2020 QuietMisdreavus
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;
use std::io;
use std::process::Command;

use tracing::{trace, debug, warn, error};

use chrono::TimeZone;

macro_rules! yeet {
    () => {
        std::process::exit(1);
    };
    ($($x:tt)*) => {
        error!($($x)*);
        yeet!();
    };
}

struct LocalPackage<'a> {
    name: &'a str,
    version: &'a str,
}

fn make_map<T, K: Eq + Hash, F: FnMut(&T) -> K>(input: Vec<T>, mut key: F) -> HashMap<K, T> {
    input.into_iter().map(|it| (key(&it), it)).collect::<HashMap<_, _>>()
}

fn args() -> clap::App<'static, 'static> {
    clap::App::new("QuietMisdreavus AUR tool")
        .version(env!("CARGO_PKG_VERSION"))
        .author("(c) 2020 QuietMisdreavus")
        .about("a personal tool to query the AUR")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .arg(clap::Arg::with_name("verbose")
            .long("verbose")
            .short("v")
            .takes_value(false)
            .multiple(true)
            .help("Emits more information. Can be given up to three times."))
        .arg(clap::Arg::with_name("quiet")
            .long("quiet")
            .short("q")
            .takes_value(false)
            .multiple(true)
            .conflicts_with("verbose")
            .help("Emits less information. Can be given once or twice."))
        .subcommand(clap::SubCommand::with_name("checkupdates")
            .about("checks the AUR for available updates to installed packages"))
        .subcommand(clap::SubCommand::with_name("search")
            .about("searches the AUR for the given string")
            .arg(clap::Arg::with_name("QUERY")
                .help("query to search for")
                .required(true)
                .index(1)))
        .subcommand(clap::SubCommand::with_name("info")
            .about("displays info about the given package")
            .arg(clap::Arg::with_name("NAME")
                .help("name of AUR package to display")
                .required(true)
                .index(1)))
        .subcommand(clap::SubCommand::with_name("generate-bash-completions")
            .about("emits a bash-completion script to stdout"))
}

fn main() -> io::Result<()> {
    let args = args().get_matches();

    let env_filter = {
        use tracing_subscriber::filter::LevelFilter;

        let verbosity = (args.occurrences_of("verbose") as i32) - (args.occurrences_of("quiet") as i32);
        let f = tracing_subscriber::EnvFilter::from_default_env();
        match verbosity {
            -1 => f.add_directive(LevelFilter::ERROR.into()),
            0 => f.add_directive(LevelFilter::WARN.into()),
            1 => f.add_directive(LevelFilter::INFO.into()),
            2 => f.add_directive(LevelFilter::DEBUG.into()),
            _ if verbosity < -1 => f.add_directive(LevelFilter::OFF.into()),
            _ if verbosity > 2 => f.add_directive(LevelFilter::TRACE.into()),
            _ => unreachable!(),
        }
    };
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();

    match args.subcommand() {
        ("checkupdates", _) => checkupdates()?,
        ("search", Some(sub_args)) => search(sub_args),
        ("info", Some(sub_args)) => info(sub_args),
        ("generate-bash-completions", _) => bashcomp(),
        _ => (), // if no subcommand was given we wouldn't have gotten here
    }

    Ok(())
}

fn checkupdates() -> io::Result<()> {
    let cmd = Command::new("pacman")
        .arg("-Qm")
        .output()?;

    if !cmd.status.success() {
        error!("pacman failed!");
        if let Ok(out) = String::from_utf8(cmd.stdout) {
            error!("stdout:");
            error!("{}", out);
        }
        if let Ok(err) = String::from_utf8(cmd.stderr) {
            error!("stderr:");
            error!("{}", err);
        }
        yeet!();
    }

    let stdout = match String::from_utf8(cmd.stdout) {
        Ok(out) => out,
        Err(err) => {
            yeet!("pacman fed non-utf8 data: {:?}", err);
        }
    };

    let mut pkglist = vec![];
    for l in stdout.lines() {
        let mut split = l.split_whitespace();
        match (split.next(), split.next()) {
            (Some(name), Some(version)) => pkglist.push(LocalPackage { name, version, }),
            _ => {
                warn!("not enough names in a line? \"{}\"", l);
            }
        }
    }
    debug!("{} foreign packages found by pacman", pkglist.len());
    let pkglist = make_map(pkglist, |p| p.name);

    let names = pkglist.keys().collect::<Vec<_>>();
    trace!("calling aurweb");
    let info = match raur::info(&names) {
        Ok(list) => list,
        Err(err) => { yeet!("aurweb returned an error: \"{}\"", err); }
    };
    debug!("{} packages returned by aurweb", info.len());
    let mut info = make_map(info, |p| p.name.clone());

    for (name, pkg) in pkglist {
        if let Some(aurpkg) = info.remove(name) {
            debug!("{} / local {} / remote {}", name, pkg.version, aurpkg.version);
            if pkg.version != aurpkg.version {
                println!("{} {} -> {}", name, pkg.version, aurpkg.version);
            }
        } else {
            warn!("--package {} was not found in AUR", name);
        }
    }

    Ok(())
}

fn search(args: &clap::ArgMatches) {
    let query = args.value_of("QUERY").expect("QUERY is required");

    debug!("search query: \"{}\"", query);

    trace!("calling aurweb");

    match raur::search(query) {
        Ok(list) => {
            for pkg in list {
                println!("{} [{}]", pkg.name, pkg.version);
                println!("    {}", pkg.description.unwrap_or_default());
            }
        }
        Err(err) => {
            yeet!("aurweb returned an error: {}", err);
        }
    }
}

fn info(args: &clap::ArgMatches) {
    let query = args.value_of("NAME").expect("NAME is required");

    debug!("info query: \"{}\"", query);

    trace!("calling aurweb");

    match raur::info(&[query]) {
        Ok(list) => {
            match list.first() {
                Some(pkg) => {
                    println!("{}", pkg.name);
                    println!("    version: {}", pkg.version);
                    println!("    AUR url: https://aur.archlinux.org/packages/{}/", pkg.name);
                    println!("    git url: https://aur.archlinux.org/{}.git", pkg.package_base);
                    println!("    upstream url: {}", pkg.url.as_deref().unwrap_or("<none>"));
                    println!("    license: {}", pkg.license.join(", "));
                    println!("    votes: {}", pkg.num_votes);
                    println!("    maintainer: {}", pkg.maintainer.as_deref().unwrap_or("<orphaned>"));
                    println!("    last update: {}",
                        chrono::Utc.timestamp(pkg.last_modified, 0).with_timezone(&chrono::Local));
                    if !pkg.groups.is_empty() {
                        println!("    group: {}", pkg.groups.join(" "));
                    }
                    if !pkg.provides.is_empty() {
                        println!("    provides: {}", pkg.provides.join(" "));
                    }
                    if !pkg.replaces.is_empty() {
                        println!("    replaces: {}", pkg.replaces.join(" "));
                    }
                    if !pkg.conflicts.is_empty() {
                        println!("    conflicts: {}", pkg.conflicts.join(" "));
                    }
                    println!("    dependencies: {}", pkg.depends.join(" "));
                    if !pkg.opt_depends.is_empty() {
                        println!("    optional: {}", pkg.opt_depends.join(" "));
                    }
                    if !pkg.make_depends.is_empty() {
                        println!("    build deps: {}", pkg.make_depends.join(" "));
                    }
                    if !pkg.check_depends.is_empty() {
                        println!("    check deps: {}", pkg.check_depends.join(" "));
                    }
                }
                None => {
                    error!("package {} was not found in the AUR", query);
                }
            }
        }
        Err(err) => {
            yeet!("aurweb returned an error: {}", err);
        }
    }
}

fn bashcomp() {
    args().gen_completions_to("qmaur", clap::Shell::Bash, &mut io::stdout());
}
