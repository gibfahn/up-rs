# up-rs

[![Latest Version](https://img.shields.io/crates/v/up-rs.svg)](https://crates.io/crates/up-rs)
[![Documentation](https://docs.rs/up-rs/badge.svg)](https://docs.rs/up-rs)

I use this to keep my machine up to date. It does a couple of different things.

See `up --help` for more details.

## Install

The binary is self-contained, you can simply download it and mark the binary as executable:

```shell
curl --create-dirs -Lo ~/bin/up https://github.com/gibfahn/up-rs/releases/latest/download/up-darwin
chmod +x ~/bin/up
```

Or if you have Cargo on your system you can install it directly:

```shell
cargo install up-rs
```

## Subcommands

### Link

```console
$ up link --from ~/code/dotfiles --to ~
```

symlinks the files in `dotfiles` into the matching directory in `~` (so `~/.config/git/config` becomes a link to
`~/code/dotfiles/.config/git/config`).

### Update

Coming soon.

Updates all the software on your machine.

## Developing

Build the documentation including internal functions with:

```console
$ cargo doc --document-private-items --open
```

<!-- TODO(gib): Finish this. -->
