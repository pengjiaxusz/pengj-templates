//! Commit gate: one Rust library for every git-hook check a repository needs.
//!
//! The crate owns four jobs that used to be spread over a node toolchain:
//! formatting staged Rust files, guarding the code language, validating
//! the commit message, and the pre-push check that catches whatever the
//! first three miss (a patch series lands through `git am`, which fires
//! neither `pre-commit` nor `commit-msg`). It has no third-party
//! dependencies, so `Cargo.lock` stays tiny and the build is a single rustc
//! invocation per file.
//!
//! Switches live in `.pengj-templates/gate.toml`, never in the manifest: the template renders
//! that file for each project and the manifest stays free to change shape.
//!
//! The crate is a library only. The gate ships inside `pengj-templates-cli`
//! and is reachable as `pengj-templates-cli gate ...`, so a project installs
//! one binary and gets the template engine, the MCP installer and the commit
//! gate at the same time. There is no separate `gate` binary to keep in sync.

pub mod cli;
pub mod commit;
pub mod config;
pub mod fmt;
pub mod git;
pub mod hooks;
pub mod lang_guard;
pub mod prepush;
pub mod scanner;
