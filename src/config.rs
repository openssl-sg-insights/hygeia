use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use failure::format_err;
use log::{debug, error};
use semver::VersionReq;

use crate::{utils, Result};

static TOOLCHAIN_FILE: &str = ".python-version";

#[derive(Debug)]
pub struct Cfg {
    version: VersionReq,
}

pub fn load_config_file() -> Result<Cfg> {
    if utils::path_exists(TOOLCHAIN_FILE) {
        Cfg::from_file(TOOLCHAIN_FILE)
    } else {
        Cfg::from_user_input()
    }
}

impl Cfg {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Cfg> {
        debug!("Reading configuration from file {:?}", path.as_ref());

        let input = File::open(path)?;
        let buffered = BufReader::new(input);

        // Read first line only
        let line = match buffered.lines().next() {
            None => Err(format_err!("File does not even contains a line"))?,
            Some(line_result) => line_result?,
        };

        Ok(Cfg {
            version: line.parse()?,
        })
    }

    pub fn from_user_input() -> Result<Cfg> {
        debug!("Reading configuration from stdin");

        let stdin = io::stdin();
        println!("Please type the Python version to use in this directory:");
        let line = match stdin.lock().lines().next() {
            None => Err(format_err!("Standard input did not contain a single line"))?,
            Some(line_result) => line_result?,
        };
        debug!("Given: {}", line);

        let version: VersionReq = line.trim().parse()?;

        if line.is_empty() {
            error!("Empty line given as input.");
            Err(format_err!("Empty line provided"))?
        } else {
            debug!("Parsed version: {}", version);
            Ok(Cfg { version })
        }
    }
}