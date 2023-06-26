//! Wrappers around executing commands.

use log::Level;
use std::ffi::OsString;
use std::fmt::Write;

/// Copy of the `duct::cmd` function that ensures we're info logging the command we're running.
pub fn cmd<T, U>(program: T, args: U) -> duct::Expression
where
    T: duct::IntoExecutablePath + Clone,
    U: IntoIterator + Clone,
    <U as IntoIterator>::Item: Into<OsString>,
{
    cmd_log(Level::Info, program, args)
}

/// Wrapper around `duct::cmd` function that lets us log the command we're running.
pub fn cmd_log<T, U>(l: log::Level, program: T, args: U) -> duct::Expression
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

    log::log!(l, "{formatted_cmd}");
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
            $crate::exec::cmd_log(log::Level::Info, $program, args)
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
            $crate::exec::cmd_log(log::Level::Debug, $program, args)
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
            $crate::exec::cmd_log(log::Level::Info, actual_program, &args)
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
                vec![OsString::from("[Dry Run])"), Into::<OsString>::into($program), $( Into::<OsString>::into($arg) ),*]
            } else {
                vec![$( Into::<OsString>::into($arg) ),*]
            };
            $crate::exec::cmd_log(log::Level::Debug, actual_program, &args)
        }
    };
}
