// FIXME: Running '/usr/bin/python2 -V' returns Python 2.7.15+, which fails parsing: ERROR pycors::settings] Failed to parse version string "2.7.15+": ParseError("Error parsing prerelease")
// FIXME: Replace 'format_err!()' with structs/enums
// FIXME: Gracefully handle errors that bubble to main
// FIXME: Add -vvv flag to control log level
// FIXME: Increase test coverage
// FIXME: Implement checksum/signature validation

use std::{
    env,
    ffi::{OsStr, OsString},
    io,
    path::PathBuf,
};

use git_testament::{git_testament, render_testament};
use lazy_static::lazy_static;
use structopt::StructOpt;

mod cache;
mod commands;
mod constants;
mod dir_monitor;
mod download;
mod os;
mod shim;
mod toolchain;
mod utils;

use crate::{commands::Command, constants::*};

use anyhow::Result;
use thiserror::Error;

git_testament!(GIT_TESTAMENT);

fn git_version() -> &'static str {
    lazy_static! {
        static ref RENDERED: String = render_testament!(GIT_TESTAMENT);
    }
    &RENDERED
}

/// Control which Python toolchain to use on a directory basis.
#[derive(StructOpt, Debug)]
#[structopt(version = git_version())]
struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    #[structopt(subcommand)]
    subcommand: Option<commands::Command>,
}

#[derive(Debug, Error)]
pub enum MainError {
    #[error("Cannot get executable's path: {0:?}")]
    Io(#[from] io::Error),
    #[error("Failed to get str representation of {0:?}")]
    Str(OsString),
    #[error("Cannot get executable's path: {0:?}")]
    ExecutablePath(PathBuf),
}

fn main() -> Result<()> {
    // Detect if running as shim as soon as possible
    let current_exe: PathBuf = env::current_exe().map_err(MainError::Io)?;
    let file_name: &OsStr = current_exe
        .file_name()
        .ok_or_else(|| MainError::ExecutablePath(current_exe.clone()))?;
    let exe = file_name
        .to_str()
        .ok_or_else(|| MainError::Str(file_name.to_os_string()))?;

    if exe.starts_with(EXECUTABLE_NAME) {
        no_shim_execution()
    } else {
        python_shim(exe)
    }
}

pub fn no_shim_execution() -> Result<()> {
    let opt = Opt::from_args();
    log::debug!("{:?}", opt);

    std::env::var("RUST_LOG").or_else(|_| -> Result<String> {
        let rust_log = format!("{}=info", EXECUTABLE_NAME);
        std::env::set_var("RUST_LOG", &rust_log);
        Ok(rust_log)
    })?;

    env_logger::init();

    if let Some(subcommand) = opt.subcommand {
        match subcommand {
            Command::Autocomplete { shell } => {
                commands::autocomplete::run(shell, &mut std::io::stdout())?;
            }
            Command::List => commands::list::run()?,
            Command::Path { version } => commands::path::run(version)?,
            Command::Version { version } => commands::version::run(version)?,
            Command::Select(version_or_path) => commands::select::run(version_or_path)?,
            Command::Install {
                from_version,
                force,
                install_extra_packages,
                select,
            } => {
                commands::install::run(from_version, force, &install_extra_packages, select)?;
            }
            Command::Run { version, command } => commands::run::run(version, &command)?,
            Command::Setup { shell } => commands::setup::run(shell)?,
        }
    }

    Ok(())
}

pub fn python_shim(command: &str) -> Result<()> {
    env_logger::init();

    let arguments: Vec<_> = env::args().collect();
    let (_, remaining_args) = arguments.split_at(1);

    shim::run(command, remaining_args)
}

#[cfg(test)]
pub mod tests {
    use std::env;

    pub fn init_logger() {
        env::var("RUST_LOG")
            .or_else(|_| -> Result<String, ()> {
                let rust_log = "debug".to_string();
                println!("Environment variable 'RUST_LOG' not set.");
                println!("Setting to: {}", rust_log);
                env::set_var("RUST_LOG", &rust_log);
                Ok(rust_log)
            })
            .unwrap();
        let _ = env_logger::try_init();
    }

    // Version is reported as "unknown" in GitHub Actions.
    // See https://github.com/nbigaouette/pycors/pull/90/checks?check_run_id=311900597
    #[test]
    #[ignore]
    fn version() {
        let crate_version = structopt::clap::crate_version!();

        // GIT_VERSION is of the shape `v0.1.7-1-g095d7f5-modified`

        // Strip out the `v` prefix
        let (v, git_version_without_v) = crate::git_version().split_at(1);

        println!("crate_version: {:?}", crate_version);
        println!("v: {}", v);
        println!("git_version_without_v: {}", git_version_without_v);

        assert_eq!(v, "v");
        assert!(git_version_without_v.starts_with(crate_version));
    }
}
