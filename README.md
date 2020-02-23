# up-rs: a tool for keeping your system up to date

I use this to keep my machine up to date. It does a couple of different things.

See `up --help` for more details.

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
