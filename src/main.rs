// FIXME: Get rid of utils::path_exists(), use std::Path::exists() instead.
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

use thiserror::Error;

use pycors::{
    commands::{self, Command},
    constants::EXECUTABLE_NAME,
    shim, Opt, Result, StructOpt,
};

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
