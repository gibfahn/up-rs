//! Docs for dot crate.
extern crate walkdir;
#[macro_use] extern crate quicli;
extern crate shellexpand;

use std::path::Path;
use std::fs;
use std::os::unix;
use std::env;

use walkdir::WalkDir;
use quicli::prelude::*;

#[derive(Debug, StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
struct Cli {
    #[structopt(flatten)]
    verbosity: Verbosity,
    #[structopt(subcommand)]
    cmd: Option<SubCommand>,
}

// TODO(gib): Change to ~.
// TODO(gib): Change to ~/backup.
#[derive(Debug, StructOpt)]
enum SubCommand {
    /// Install or update everything on your computer.
    #[structopt(name = "update")]
    Update {
    },

    /// Symlink your dotfiles into your config directory.
    #[structopt(name = "link")]
    Link {
        /// Path where your dotfiles are kept (hopefully in source control)
        #[structopt(default_value="./dotfiles")]
        from_dir: String,
        /// Path to link them to (defaults to $HOME)."
        #[structopt(default_value="~/tmp/dot")]
        to_dir: String,
        #[structopt(default_value="~/tmp/dot/backup")]
        backup_dir: String,
    },
}

main!(|args: Cli, log_level: verbosity| {
    info!("Starting dot.");
    trace!("Received args: {:#?}", args.cmd);
    match args.cmd {
        Some(SubCommand::Update{}) => {
        },
        Some(SubCommand::Link{from_dir, to_dir, backup_dir}) => {
            println!("Current directory: {:?}", env::current_dir().unwrap());
            link(&from_dir, &to_dir, &backup_dir)?;
        },
        None => {
            println!("Use -h or --help for the usage args.");
        },
    }
    trace!("Finished dot.");
});

/// Symlink everything from to_dir (default: ./dotfiles/) into
/// $FROMPATH (default: $HOME). Anything that would be overwritten is copied into
/// $BACKUP (default: $FROMPATH/backup/).
///
/// Basically you put your dotfiles in ./dotfiles/, in the same structure they
/// were in relative to $HOME. Then if you want to edit your .bashrc (for
/// example) you just edit ~/.bashrc, and as it's a symlink it'll actually edit
/// dotfiles/.bashrc. Then you can add and commit that change.
fn link(from_dir: &str, to_dir: &str, backup_dir: &str) -> Result<()> {
    let from_dir   = shellexpand::tilde(&from_dir).to_string();
    let to_dir     = shellexpand::tilde(&to_dir).to_string();
    let backup_dir = shellexpand::tilde(&backup_dir).to_string();

    let from_dir = Path::new(&from_dir);
    let to_dir = Path::new(&to_dir);
    let backup_dir = Path::new(&backup_dir);

    fs::create_dir_all(from_dir)?;
    fs::create_dir_all(to_dir)?;
    fs::create_dir_all(backup_dir)?;

    let from_dir = from_dir.canonicalize()?;
    let to_dir = to_dir.canonicalize()?;
    let backup_dir = backup_dir.canonicalize()?;
    info!("Linking to {:?}", to_dir);

    println!("To dir: {:?}", to_dir);
    assert!(to_dir.exists());
    debug!("to_dir contents: {:#?}", fs::read_dir(&to_dir).unwrap().filter_map(|d| d.ok()).map(|x| x.path()).collect::<Vec<_>>());

    for from_path in WalkDir::new(&from_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|f| !f.file_type().is_dir())
    {
        let rel_path = from_path.path().strip_prefix(&from_dir).unwrap();
        let to_path = to_dir.join(rel_path);

        info!("Linking: {}", rel_path.display());
        fs::create_dir_all(to_path.parent().unwrap())?;

        if to_path.exists() {
            let to_path_file_type = to_path.symlink_metadata()?.file_type();
            if to_path_file_type.is_symlink() {
                match to_path.read_link() {
                    Ok(existing_link) => {
                        if existing_link == from_path.path() {
                            debug!("Link at {:?} already points to {:?}, skipping.", to_path, existing_link);
                            continue;
                        } else {
                            warn!("Link at {:?} points to {:?}, changing to {:?}.", to_path, existing_link, from_path.path());
                            fs::remove_file(&to_path)?;
                        }
                    },
                    Err(e) => {
                        error!("read_link returned error {:?} for {:?}", e, to_path);
                        unimplemented!();
                    }
                }
            } else if to_path_file_type.is_dir() {
                warn!("Expected file or link at {:?}, found directory, moving to {:?}", to_path, backup_dir);
                let backup_path = backup_dir.join(rel_path);
                fs::create_dir_all(&backup_path)?;
                fs::rename(&to_path, &backup_path)?;
            } else if to_path_file_type.is_file() {
                warn!("Existing file at {:?}, moving to {:?}", to_path, backup_dir);
                let backup_path = backup_dir.join(rel_path);
                fs::create_dir_all(backup_path.parent().unwrap())?;
                fs::rename(&to_path, &backup_path)?;
            }
        }
        info!("Linking: {:?} -> {:?}", from_path, to_path);
        unix::fs::symlink(from_path.path(), &to_path)?;
    }

    // TODO(gib): If backup dir empty, remove it, else catch specific err.
    if let Err(err) = fs::remove_dir(backup_dir) {
        info!("Backup dir remove err: {:?}", err);
    }

    debug!("to_dir final contents: {:?}", fs::read_dir(&to_dir).unwrap().collect::<Vec<_>>());

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
