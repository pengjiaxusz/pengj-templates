//! Commit gate: one Rust binary for every git-hook check a repository needs.
//!
//! The crate owns three jobs that used to be spread over a node toolchain:
//! formatting staged Rust files, guarding the code language, and validating
//! the commit message. It has no third-party dependencies, so `Cargo.lock`
//! stays tiny and the build is a single rustc invocation per file.
//!
//! Switches live in `.gate.toml`, never in the manifest: the template renders
//! that file for each project and the manifest stays free to change shape.
//!
//! The crate is both a library and a binary. The library lets the template
//! CLI expose the same checks under `pengj-templates-cli gate ...`, so the
//! gate travels with the tool a project already installs.

pub mod cli;
pub mod commit;
pub mod config;
pub mod fmt;
pub mod git;
pub mod hooks;
pub mod lang_guard;
pub mod scanner;
