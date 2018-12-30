use std::env;

use failure::format_err;
use log::{debug, error};

mod config;
mod settings;
mod utils;

use crate::config::load_config_file;
use crate::settings::load_settings_file;

pub type Result<T> = std::result::Result<T, failure::Error>;

fn main() -> Result<()> {
    env_logger::init();

    match env::args().next() {
        None => {
            error!("Cannot get first argument.");
            Err(format_err!("Cannot get first argument"))?
        }
        Some(arg) => {
            if arg.ends_with("pycors") {
                debug!("Running pycors");
                pycors()
            } else {
                debug!("Running a Python shim");
                python_shim()
            }
        }
    }
}

fn python_shim() -> Result<()> {
    unimplemented!()
}

fn pycors() -> Result<()> {
    let settings = load_settings_file()?;
    let cfg = load_config_file()?;

    debug!("settings: {:?}", settings);
    debug!("cfg: {:?}", cfg);

    Ok(())
}
