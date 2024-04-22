//! Wrappers around executing commands.

use crate::log;
use camino::Utf8Path;
use duct::Expression;
use std::ffi::OsString;
use std::fmt::Write;
use std::io;
use std::process::Output;
use tracing::Level;

/// Copy of the `duct::cmd` function that ensures we're info logging the command we're running.
pub fn cmd<T, U>(program: T, args: U) -> Expression
where
    T: duct::IntoExecutablePath + Clone,
    U: IntoIterator + Clone,
    <U as IntoIterator>::Item: Into<OsString>,
{
    cmd_log(Level::INFO, program, args)
}

/// Wrapper around `duct::cmd` function that lets us log the command we're running.
pub fn cmd_log<T, U>(l: Level, program: T, args: U) -> Expression
where
    T: duct::IntoExecutablePath + Clone,
    U: IntoIterator + Clone,
    U::Item: Into<OsString>,
{
    let mut formatted_cmd = format!(
        "Running command: {program}",
        program = shell_escape::escape(program.clone().to_executable().to_string_lossy())
    );
    for arg in args.clone() {
        write!(
            formatted_cmd,
            " {arg}",
            arg = shell_escape::escape(arg.into().to_string_lossy())
        )
        .unwrap();
    }

    log!(l, "{formatted_cmd}");

    duct::cmd(program, args)
}

/// Copy of the `duct::cmd!` macro that ensures we're logging the command we're running at the
/// 'info' level (logged by default).
#[macro_export]
macro_rules! cmd {
    ( $program:expr $(, $arg:expr )* $(,)? ) => {
        {
            use std::ffi::OsString;
            let args: &[OsString] = &[$( Into::<OsString>::into($arg) ),*];
            $crate::exec::cmd_log(tracing::Level::INFO, $program, args)
        }
    };
}

/// Copy of the `duct::cmd!` macro that ensures we're logging the command we're running at the debug
/// level (not logged by default).
#[macro_export]
macro_rules! cmd_debug {
    ( $program:expr $(, $arg:expr )* $(,)? ) => {
        {
            use std::ffi::OsString;
            let args: &[OsString] = &[$( Into::<OsString>::into($arg) ),*];
            $crate::exec::cmd_log(tracing::Level::DEBUG, $program, args)
        }
    };
}

/// Copy of the `duct::cmd!` macro that skips running the command if the `dry_run` boolean is
/// `true`. Logs the command to be run at the `Info` level.
#[macro_export]
macro_rules! cmd_if_wet {
    ( $dry_run:expr, $program:expr $(, $arg:expr )* $(,)? ) => {
        {
            use std::ffi::OsString;
            let mut actual_program = $program;
            let args: Vec<OsString> = if $dry_run {
                actual_program = "true";
                vec![OsString::from("[Dry Run]"), Into::<OsString>::into($program), $( Into::<OsString>::into($arg) ),*]
            } else {
                vec![$( Into::<OsString>::into($arg) ),*]
            };
            $crate::exec::cmd_log(tracing::Level::INFO, actual_program, &args)
        }
    };
}

/// Copy of the `duct::cmd!` macro that skips running the command if the `dry_run` boolean is
/// `true`. Logs the command to be run at the `Debug` level.
#[macro_export]
macro_rules! cmd_debug_if_wet {
    ( $dry_run:expr, $program:expr $(, $arg:expr )* $(,)? ) => {
        {
            use std::ffi::OsString;
            let mut actual_program = $program;
            let args: Vec<OsString> = if $dry_run {
                actual_program = "true";
                vec![OsString::from("[Dry Run]"), Into::<OsString>::into($program), $( Into::<OsString>::into($arg) ),*]
            } else {
                vec![$( Into::<OsString>::into($arg) ),*]
            };
            $crate::exec::cmd_log(tracing::Level::DEBUG, actual_program, &args)
        }
    };
}

/// Trait to allow retrying a command a number of times.
pub trait LivDuct {
    /**
    Run with the stdout sent to wherever `stdout_fn` points to.

    You should normally use this instead of the `.run()` function, to make sure you don't
    accidentally write stdout to liv's stdout, as this pollutes stdout, and may cause
    liv commands to fail for users.
    */
    fn run_with(&self, stdout_fn: fn(&Expression) -> Expression) -> io::Result<Output>;

    /**
    Run with the stdout sent to path `path`.

    Alternative to the `.run_with()` function as this takes a path argument.
    */
    fn run_with_path(&self, path: &Utf8Path) -> io::Result<Output>;
}

impl LivDuct for Expression {
    /// Run with the stdout sent to wherever `stdout_fn` points to.
    fn run_with(&self, stdout_fn: fn(&Expression) -> Expression) -> io::Result<Output> {
        // This method is blocked elsewhere to force people to use the `.run_with*()` functions.
        // So we need to be able to use it here.
        #[allow(clippy::disallowed_methods)]
        stdout_fn(self).run()
    }

    /// Run with the stdout sent to wherever `stdout_fn` points to.
    fn run_with_path(&self, path: &Utf8Path) -> io::Result<Output> {
        // This method is blocked elsewhere to force people to use the `.run_with*()` functions.
        // So we need to be able to use it here.
        #[allow(clippy::disallowed_methods)]
        self.stdout_path(path).run()
    }
}
