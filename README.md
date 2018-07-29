# dot: a tool for managing dotfiles

I use this to keep my machine up to date. It does a couple of different things.

See `dot --help` for more details.

## Subcommands

### Link

```console
$ dot link ~/code/dotfiles ~
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
