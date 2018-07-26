//! Docs for dot crate.
extern crate walkdir;
#[macro_use] extern crate quicli;
extern crate shellexpand;

use std::fs;
use std::os::unix;
use std::path::{Path, PathBuf};

use quicli::prelude::*;
use walkdir::WalkDir;

#[derive(Debug, StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
struct Cli {
    #[structopt(flatten)]
    verbosity: Verbosity,
    #[structopt(subcommand)]
    cmd: Option<SubCommand>,
}

#[derive(Debug, StructOpt)]
enum SubCommand {
    /// Install or update everything on your computer.
    #[structopt(name = "update")]
    Update {},

    /// Symlink your dotfiles into your config directory.
    #[structopt(name = "link")]
    Link {
        /// Path where your dotfiles are kept (hopefully in source control)
        #[structopt(default_value = "~/code/dotfiles")]
        from_dir: String,
        /// Path to link them to (defaults to $HOME)."
        // TODO(gib): Change to ~.
        #[structopt(default_value = "~/tmp/dot")]
        to_dir: String,
        // TODO(gib): Change to ~/backup.
        #[structopt(default_value = "~/tmp/dot/backup")]
        backup_dir: String,
    },
}

main!(|args: Cli, log_level: verbosity| {
    trace!("Starting dot.");
    trace!("Received args: {:#?}", args.cmd);
    match args.cmd {
        Some(SubCommand::Update {}) => {
            update();
        }
        Some(SubCommand::Link {
            from_dir,
            to_dir,
            backup_dir,
        }) => {
            link(&from_dir, &to_dir, &backup_dir)?;
        }
        None => {
            println!("Use -h or --help for the usage args.");
        }
    }
    trace!("Finished dot.");
});

fn update() {
    // TODO(gib): Implement update function:
    // TODO(gib): Need a graph of toml files, each one representing a component.
    // TODO(gib): Need a root file that can set variables (e.g. boolean flags).
    // TODO(gib): Everything has one (or more?) parents (root is the root).
    // TODO(gib): Need a command to show the tree and dependencies.
    // TODO(gib): If fixtures are needed can link to files or scripts.
    // TODO(gib): Should files be stored in ~/.config/dot ?
}

/// Symlink everything from to_dir (default: ~/code/dotfiles/) into
/// from_dir (default: ~). Anything that would be overwritten is copied into
/// backup_dir (default: ~/backup/).
///
/// Basically you put your dotfiles in ~/code/dotfiles/, in the same structure they
/// were in relative to ~. Then if you want to edit your .bashrc (for
/// example) you just edit ~/.bashrc, and as it's a symlink it'll actually edit
/// ~/dotfiles/.bashrc. Then you can add and commit that change in ~/dotfiles.
fn link(from_dir: &str, to_dir: &str, backup_dir: &str) -> Result<()> {
    // Expand ~, this is only used for the default options, if the user passes them as
    // explicit args then they will be expanded by the shell.
    let from_dir = PathBuf::from(shellexpand::tilde(&from_dir).to_string());
    let to_dir = PathBuf::from(shellexpand::tilde(&to_dir).to_string());
    let backup_dir = PathBuf::from(shellexpand::tilde(&backup_dir).to_string());

    // TODO(gib): Test this works.
    let from_dir = from_dir.canonicalize()?;
    assert!(&from_dir.is_dir(), "From directory (dotfile directory) {:?} should exist.", &from_dir);
    let to_dir = to_dir.canonicalize()?;
    // TODO(gib): Test this works.
    assert!(&to_dir.is_dir(), "To directory (home directory) {:?} should exist.", &to_dir);

    // Create the backup dir if it doesn't exist.
    fs::create_dir_all(&backup_dir)?;
    let backup_dir = backup_dir.canonicalize()?;
    // TODO(gib): Test this works.
    assert!(&backup_dir.is_dir(), "Backup directory {:?} should exist.", &backup_dir);

    info!("Linking from {:?} to {:?}.", from_dir, to_dir);
    debug!(
        "to_dir contents: {:?}",
        fs::read_dir(&to_dir)
            .unwrap()
            .filter_map(|d| d.ok())
            .map(|x| x.path().strip_prefix(&to_dir).unwrap().to_path_buf())
            .collect::<Vec<_>>()
    );

    // TODO(gib): add this to vimrc.
    // let g:rustfmt_autosave = 1
    for from_path in WalkDir::new(&from_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|f| !f.file_type().is_dir())
    {
        let rel_path = from_path.path().strip_prefix(&from_dir).unwrap();
        let to_path = to_dir.join(rel_path);

        info!("Linking: {}", rel_path.display());
        fs::create_dir_all(to_path.parent().unwrap()).or_else(|_err| {
            info!("Failed to create parent dir, walking up the tree to see if there's a file that needs to become a directory.");
            for path in rel_path.ancestors().skip(1).filter(|p| p != &Path::new("")) {
                debug!("Checking path {:?}", path);
                let abs_path = to_dir.join(path);
                if abs_path.exists() || abs_path.symlink_metadata().is_ok() {
                    assert!(!abs_path.is_dir());
                    warn!(
                        "File will be overwritten by parent directory of link.\n  \
                         File: {:?}\n  Link: {:?}",
                        &abs_path, &to_path
                    );
                    if abs_path.is_file() {
                        info!("Parent path: {:?}", &path.parent().unwrap());
                        let parent_path_opt = &path.parent();
                        if parent_path_opt.is_some() {
                            let parent_path = parent_path_opt.unwrap();
                            info!("Path: {:?}, parent: {:?}", path, parent_path);
                            if parent_path != Path::new("") {
                                fs::create_dir_all(&backup_dir.join(parent_path))?;
                            }
                            let backup_path = backup_dir.join(path);
                            info!("Moving file: {:?} -> {:?}", &abs_path, &backup_path);
                            fs::rename(&abs_path, backup_path)?;
                        }
                    } else {
                        info!("Removing link: {:?}", abs_path);
                        fs::remove_file(abs_path)?;
                    }
                }
            }
            fs::create_dir_all(to_path.parent().unwrap())
        })?;

        if to_path.exists() {
            let to_path_file_type = to_path.symlink_metadata()?.file_type();
            if to_path_file_type.is_symlink() {
                match to_path.read_link() {
                    Ok(existing_link) => {
                        if existing_link == from_path.path() {
                            debug!(
                                "Link at {:?} already points to {:?}, skipping.",
                                to_path, existing_link
                            );
                            continue;
                        } else {
                            warn!(
                                "Link at {:?} points to {:?}, changing to {:?}.",
                                to_path,
                                existing_link,
                                from_path.path()
                            );
                            fs::remove_file(&to_path)?;
                        }
                    }
                    Err(e) => {
                        error!("read_link returned error {:?} for {:?}", e, to_path);
                        unimplemented!();
                    }
                }
            } else if to_path_file_type.is_dir() {
                warn!(
                    "Expected file or link at {:?}, found directory, moving to {:?}",
                    to_path, backup_dir
                );
                let backup_path = backup_dir.join(rel_path);
                fs::create_dir_all(&backup_path)?;
                fs::rename(&to_path, &backup_path)?;
            } else if to_path_file_type.is_file() {
                warn!("Existing file at {:?}, moving to {:?}", to_path, backup_dir);
                let backup_path = backup_dir.join(rel_path);
                fs::create_dir_all(backup_path.parent().unwrap())?;
                fs::rename(&to_path, &backup_path)?;
            }
        } else if to_path.symlink_metadata().is_ok() {
            warn!(
                "Removing existing broken link ({:?} -> {:?})",
                &to_path,
                &to_path.read_link()?
            );
            fs::remove_file(&to_path)?;
        }
        info!("Linking: {:?} -> {:?}", from_path, to_path);
        unix::fs::symlink(from_path.path(), &to_path)?;
    }

    // TODO(gib): If backup dir empty, remove it, else catch specific err.
    if let Err(err) = fs::remove_dir(backup_dir) {
        info!("Backup dir remove err: {:?}", err);
    }

    debug!(
        "to_dir final contents: {:?}",
        fs::read_dir(&to_dir).unwrap().collect::<Vec<_>>()
    );

    Ok(())
}

// TODO(gib): Implement this.
// : ${FROMPATH:="$HOME"} # Where you keep your dotfiles, overwrite if necessary.
// : ${BACKUP:="$FROMPATH/backup"} # Place to back up old files.
// mkdir -p "$BACKUP"

// if [ -z "$TOPATH" ]; then
//   cd "$(dirname $0)/dotfiles/"
// else
//   cd "$TOPATH"
// fi
// : ${TOPATH:="$PWD"}

// printf "❯❯❯ Updating dotfile symlinks (linking from path: $TOPATH)\n\n"

// for FILE in $(find . -type f -o -type l | sed 's|./||'); do
//   mkdir -p "$FROMPATH/$(dirname $FILE)"
//   if [ -d "$FROMPATH/$FILE" -a ! -L "$FROMPATH/$FILE" ]; then # Directory.
//     printf "${RED}DIRSKIP: $FROMPATH/$FILE is a directory!${NC}\n" # This shouldn't happen.
//     continue
//   elif [ -L "$FROMPATH/$FILE" ]; then # Symlink.
//     if [ "$(ls -l $FROMPATH/$FILE | awk '{print $NF}')" = "$TOPATH/$FILE" ]; then
//       printf "${BBLACK}SKIP: $FROMPATH/$FILE already points to $TOPATH/$FILE.${NC}\n"
//       continue
//     fi
//     echo "CHANGE: $FROMPATH/$FILE $(ls -l $FROMPATH/$FILE | awk '{print $NF}') \
//     -> $TOPATH/$FILE\n"
//     mkdir -p "$BACKUP/$(dirname $FILE)"
//     rm "$FROMPATH/$FILE" "$BACKUP/$FILE"
//   elif [ -e "$FROMPATH/$FILE" ]; then # File.
//     echo "MOVE: $FROMPATH/$FILE exists, moving to $BACKUP/$FILE"
//     mkdir -p "$BACKUP/$(dirname $FILE)"
//     mv "$FROMPATH/$FILE" "$BACKUP/$FILE"
//   else # Nothing there.
//     echo "LINK: $FROMPATH/$FILE -> $TOPATH/$FILE"
//   fi
//   ln -s "$TOPATH/$FILE" "$FROMPATH/$FILE"
// done

// [ "$(ls -A $BACKUP)" ] || rm -r "$BACKUP" # Clean up backup folder if empty
