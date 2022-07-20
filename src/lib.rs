use std::fmt::format;
use std::fs::File;
use std::{fs, io};
use std::path::{Path, PathBuf};
use chrono::{SecondsFormat, Utc};

use log::{LevelFilter, SetLoggerError};
use openssl::hash::{DigestBytes, Hasher, MessageDigest};
use serde::{Deserialize, Serialize};
use simplelog::{ColorChoice, CombinedLogger, Config, TerminalMode, TermLogger, WriteLogger};
use tokio::time::Instant;

pub use error::Error;

pub mod error;
pub mod updater;

#[derive(Serialize, Deserialize)]
/// Represents a file in the Info struct
pub struct FileInfo {
    /// The path of the file relative to the root of the program
    pub path: String,
    /// The hash of the file (using Crockford representation)
    pub hash: String,
}

#[derive(Serialize, Deserialize)]
/// Represents the JSON sent by the server to compute which files must be updated
pub struct Info {
    /// Url of the root of the program on the remote server
    pub base_url: String,
    /// Algorithm to generate hashes
    pub algorithm: String,
    /// Vector of the file info of all files
    pub files: Vec<FileInfo>,
}

/// Compute the hash of a file
/// # Arguments
/// * 'path' - The path of the file to hash
/// * 'digest' - The hash algorithm to use
pub fn hash_file(path: &PathBuf, digest: MessageDigest) -> Result<DigestBytes, Error> {
    let mut hasher = Hasher::new(digest)?;
    let mut file = File::open(path)?;
    io::copy(&mut file, &mut hasher)?;
    hasher.finish().map_err(Error::from)
}

pub fn scan_dir(path: PathBuf) -> Result<Vec<PathBuf>, Error> {
    if path.is_dir() {
        path.read_dir()?
            .map(|d|
                scan_dir(d.map_err(|_| Error::Other("".to_string()))?.path()))
            .collect::<Result<Vec<Vec<PathBuf>>, Error>>()
            .map(|v|
                v.into_iter()
                    .flatten()
                    .collect())
    } else {
        Ok(Vec::from([path]))
    }
}

pub fn init_logger() -> Result<(), SetLoggerError> {
    let dir  =Path::new("./logs/");
    if !dir.exists() {
        fs::create_dir(dir).expect("Could not create log dir");
    }

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Trace,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create("./logs/latest.log").expect("Could not create log file"),
        ),
        WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create(format!("./logs/{}.log", Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true))).expect("Could not create log file"),
        ),
    ])
}