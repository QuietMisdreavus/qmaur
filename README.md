# QuietMisdreavus AUR tool

this is a little sketch to have a read-only tool to query the [AUR] from the command-line. primarily
i wanted an analog to the `checkupdates` tool that `pacman` ships to check if there are updates to
system packages, but i didn't want to install a full-blown "AUR helper" that built and installed
stuff for me.

[AUR]: https://aur.archlinux.org

there are three primary commands here:

* `qmaur checkupdates` checks the AUR for the currently-installed "foreign" packages to see if the
  most recently version is different from the one that is currently installed. if everything is up
  to date, it prints nothing. (it's `checkupdates` for the AUR, like i wanted in the intro)
* `qmaur search thing` searches the AUR index for "thing" and prints the results out.
* `qmaur info package` looks up `package` on the AUR and prints its info out.

i also included `qmaur generate-bash-completions` because i don't want to type everything out if i'm
looking for things by hand.

to set this up, use the following commands after cloning the repo:

```
cargo install --path .
# optional: add bash-completions to the user bash-completions dir
mkdir -p ~/.local/share/bash-completion/completions
qmaur generate-bash-completions > ~/.local/share/bash-completion/completions/qmaur
```
