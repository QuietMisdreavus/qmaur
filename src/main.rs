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

use std::io;
use std::process::Command;

macro_rules! yeet {
    () => {
        std::process::exit(1);
    };
    ($($x:tt)*) => {
        eprintln!($($x)*);
        yeet!();
    };
}

struct Package<'a> {
    name: &'a str,
    version: &'a str,
}

fn main() -> io::Result<()> {
    let cmd = Command::new("pacman")
        .arg("-Qm")
        .output()?;

    if !cmd.status.success() {
        eprintln!("pacman failed!");
        if let Ok(out) = String::from_utf8(cmd.stdout) {
            eprintln!("stdout:");
            eprintln!("{}", out);
        }
        if let Ok(err) = String::from_utf8(cmd.stderr) {
            eprintln!("stderr:");
            eprintln!("{}", err);
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
            (Some(name), Some(version)) => pkglist.push(Package { name, version, }),
            _ => {
                eprintln!("--error: not enough names in a line? \"{}\"", l);
            }
        }
    }

    for pkg in pkglist {
        println!("{} {}", pkg.name, pkg.version);
    }

    Ok(())
}
