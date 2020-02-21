//! Module to hold all the tests and the common parts so that you don't get dead code warnings for
//! common utils.

// Common utils.
mod common;

// Test config handling and parsing.
mod config;

// Basic check that `up --help` works.
mod help;

// Test `up link`.
mod link;

// Make sure `cargo clippy` and `cargo fmt` were run.
mod z_style;
