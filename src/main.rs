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

fn main() -> io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

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
