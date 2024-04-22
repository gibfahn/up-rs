//! Common functions that are used by other tests.
use assert_cmd::Command;
use camino::Utf8Path;
use camino::Utf8PathBuf;
use color_eyre::eyre::ensure;
use color_eyre::eyre::OptionExt;
use color_eyre::Result;
pub use pretty_assertions;
pub use pretty_assertions_sorted;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::os::unix;
use walkdir::WalkDir;

pub mod ensure_utils;

/// Returns the path to the root of the project (the {crate}/ folder).
fn crate_root() -> Result<Utf8PathBuf> {
    // Directory of the testutils Cargo.toml i.e. {crate}/tests/testutils/
    let mut project_dir = Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Pop up to tests/ dir.
    ensure!(project_dir.pop());
    // Pop up to crate dir.
    ensure!(project_dir.pop());
    Ok(project_dir)
}

/// Returns a new command starting with /path/to/{binary} (add args as needed).
pub fn crate_binary_cmd(binary_name: &str, temp_dir: &Utf8Path) -> Result<Command> {
    let mut cmd = Command::cargo_bin(binary_name)?;
    // Set temp dir to be inside our test's temp dir.
    cmd.env("TMPDIR", temp_dir.join(format!("{binary_name}_temp_dir")));
    // Always print colours, even when output is not a tty.
    cmd.env("RUST_LOG_STYLE", "always");
    // Show backtrace on exit, nightly only for now.
    // https://github.com/rust-lang/rust/issues/53487
    cmd.env("RUST_BACKTRACE", "1");
    cmd.args(
        [
            "--log-level=trace",
            "--up-dir",
            temp_dir.join("up-rs").as_str(),
            "--color=always",
        ]
        .iter(),
    );
    Ok(cmd)
}

/// Extensions to `assert_cmd` functions.
pub trait AssertCmdExt {
    /**
    `assert_cmd`'s assert functions truncate the stdout and stderr.
    This is painful in CI, so add a function to always print them in CI.
    Refs: <https://github.com/assert-rs/assert_cmd/issues/180>
    */
    fn eprint_stdout_stderr(self) -> Self;
}

impl AssertCmdExt for assert_cmd::assert::Assert {
    /**
    `assert_cmd`'s assert functions truncate the stdout and stderr.
    This is painful in CI, so add a function to always print them in CI.

    In general instead of `.success()?` and `.stderr()` we use `.try_success()?` and
    `.try_stderr()?`. This is mostly to remind the author to use this method before
    asserting.

    Refs: <https://github.com/assert-rs/assert_cmd/issues/180>
    */
    fn eprint_stdout_stderr(self) -> Self {
        let output = self.get_output();
        eprintln!(
            "LIV COMMAND STDOUT:\n-------------------\n<<<<\n{stdout}\n>>>>\n\nLIV COMMAND \
             STDERR:\n-------------------\n<<<<\n{stderr}\n>>>>",
            stdout = String::from_utf8_lossy(&output.stdout),
            stderr = String::from_utf8_lossy(&output.stderr),
        );
        self
    }
}
/// Returns the path to the tests/fixtures directory (relative to the crate
/// root).
fn fixtures_dir() -> Result<Utf8PathBuf> {
    Ok(crate_root()?.join("tests/fixtures"))
}

/// Returns the subdirectory of the tests/fixtures directory for the current function (relative to
/// the crate root).
pub fn fixtures_subdir(function_path: &str) -> Result<Utf8PathBuf> {
    Ok(fixtures_dir()?.join(subdir_path(function_path)))
}

/// Returns the path to a temporary directory for your test (OS tempdir + test
/// function path). Cleans the directory if it already exists.
///
/// ```rust
/// # fn test_requiring_tempdir() -> color_eyre::Result<()> {
/// let temp_dir = testutils::temp_dir(testutils::function_path!())?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Fails if any of the underlying file system operations fail.
pub fn temp_dir(binary_name: &str, function_path: &str) -> Result<Utf8PathBuf> {
    let os_temp_dir = Utf8PathBuf::try_from(env::temp_dir())?.canonicalize_utf8()?;
    let mut temp_dir = os_temp_dir.clone();
    temp_dir.push(format!("{binary_name}_test_tempdirs"));
    temp_dir.push(function_path.replace("::", "/"));
    ensure!(temp_dir.starts_with(&os_temp_dir));
    let remove_dir_result = fs::remove_dir_all(&temp_dir);
    if matches!(&remove_dir_result, Err(e) if e.kind() != ErrorKind::NotFound) {
        remove_dir_result?;
    }
    ensure!(!temp_dir.exists());
    fs::create_dir_all(&temp_dir)?;
    Ok(temp_dir)
}

fn subdir_path(function_path: &str) -> String {
    function_path
        .replace("::", "/")
        // Integration test function paths seem to end with this now.
        .trim_end_matches(r"{{closure}}")
        .to_owned()
}

/// Expands to the current function path.
#[macro_export]
macro_rules! function_path {
    () => {{
        // Okay, this is ugly, I get it. However, this is the best we can get on a stable rust.
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name
            // The f() above gives us the trailing `f` in the path.
            .trim_end_matches("::f")
            // If we use test_log::test for tests, we get a trailing `test_impl` in the path.
            .trim_end_matches("::test_impl")
    }};
}

/// Copy everything in `from_dir` into `to_dir` (including broken links).
///
/// # Errors
///
/// Fails if any of the underlying file system operations fail.
pub fn copy_all(from_dir: &Utf8Path, to_dir: &Utf8Path) -> Result<()> {
    println!("Copying everything in {from_dir} to {to_dir}");
    ensure!(
        from_dir.exists(),
        "Cannot copy from non-existent directory {from_dir}.",
    );
    for from_path in WalkDir::new(from_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
    {
        let from_path_metadata = from_path.metadata()?;
        let from_path = <&Utf8Path>::try_from(from_path.path())?;

        let rel_path = from_path.strip_prefix(from_dir)?;
        println!("Copying: {rel_path}");
        let to_path = to_dir.join(rel_path);

        let file_type = from_path_metadata.file_type();
        fs::create_dir_all(to_path.parent().ok_or_eyre("Path was root?")?)?;
        if file_type.is_dir() {
            fs::create_dir(to_path)?;
        } else if file_type.is_symlink() {
            unix::fs::symlink(fs::read_link(from_path)?, to_path)?;
        } else if file_type.is_file() {
            fs::copy(from_path, to_path)?;
        }
    }
    Ok(())
}

/**
Ensures that two expressions are equal to each other (using [`PartialEq`]).
Uses pretty_assertions to generate a pretty diff.
We want to use ensure to return a result to make sure that destructors run, which doesn't happen after a panic.

On error, this macro will print a diff derived from [`Debug`] representation of
each value.

This is a drop in replacement for [`core::assert_eq!`] except that it returns a result.
You can provide a custom error message if desired.

# Examples

```
use testutils::ensure_eq;

let a = 3;
let b = 1 + 2;
ensure_eq!(a, b);

ensure_eq!(a, b, "we are testing addition with {} and {}", a, b);

# Ok(())
```
*/
#[macro_export]
macro_rules! ensure_eq {
    ($left:expr, $right:expr$(,)?) => ({
        $crate::ensure_eq!(@ $left, $right, "", "");
    });
    ($left:expr, $right:expr, $($arg:tt)*) => ({
        $crate::ensure_eq!(@ $left, $right, ": ", $($arg)+);
    });
    (@ $left:expr, $right:expr, $maybe_colon:expr, $($arg:tt)*) => ({
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    ::color_eyre::eyre::bail!("assertion failed: `(left == right)`{}{}\
                       \n\
                       \n{}\
                       \n",
                       $maybe_colon,
                       format_args!($($arg)*),
                       $crate::pretty_assertions::Comparison::new(left_val, right_val)
                    )
                }
            }
        }
    });
}

/**
This is a wrapper with similar functionality to [`ensure_eq`], however, the
[`Debug`] representation is sorted to provide deterministic output.

Copied from [`pretty_assertions_sorted::assert_eq_sorted`]

Not all [`Debug`] representations are sortable yet and this doesn't work with
custom [`Debug`] implementations that don't conform to the format that #[derive(Debug)]
uses, eg. `fmt.debug_struct()`, `fmt.debug_map()`, etc.

Don't use this if you want to test the ordering of the types that are sorted, since
sorting will clobber any previous ordering.

*/
#[macro_export]
macro_rules! ensure_eq_sorted {
    ($left:expr, $right:expr$(,)?) => ({
        $crate::ensure_eq_sorted!(@ $left, $right, "", "");
    });
    ($left:expr, $right:expr, $($arg:tt)*) => ({
        $crate::ensure_eq_sorted!(@ $left, $right, ": ", $($arg)+);
    });
    (@ $left:expr, $right:expr, $maybe_semicolon:expr, $($arg:tt)*) => ({
        match (&($left), &($right)) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    // We create the comparison string outside the panic! call
                    // because creating the comparison string could panic itself.
                    let comparison_string = $crate::pretty_assertions_sorted::Comparison::new(
                        &$crate::pretty_assertions_sorted::SortedDebug::new(left_val),
                        &$crate::pretty_assertions_sorted::SortedDebug::new(right_val)
                    ).to_string();
                    ::color_eyre::eyre::bail!("assertion failed: `(left == right)`{}{}\
                       \n\
                       \n{}\
                       \n",
                       $maybe_semicolon,
                       format_args!($($arg)*),
                       comparison_string,
                    )
                }
            }
        }
    });
}
