use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use chrono::{SecondsFormat, Utc};
use log::{LevelFilter, SetLoggerError};
use ring::digest::{Algorithm, Context, Digest, SHA256, SHA384, SHA512, SHA512_256};
use serde::{Deserialize, Serialize};
use simplelog::{ColorChoice, CombinedLogger, Config, TerminalMode, TermLogger, WriteLogger};

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
    /// Vector of all the files/dirs that won't be modified
    pub ignored_files: Vec<PathBuf>,
}

/// Compute the hash of a file
/// # Arguments
/// * 'path' - The path of the file to hash
/// * 'digest' - The hash algorithm to use
pub fn hash_file(path: &PathBuf, algo: &'static Algorithm) -> Result<Digest, Error> {
    let mut file = File::open(path)?;
    let mut context = Context::new(algo);
    let mut buffer = [0; 1024];

    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    Ok(context.finish())
}

pub fn scan_dir(path: PathBuf, ignored: &Vec<PathBuf>) -> Result<Vec<PathBuf>, Error> {
    if ignored.contains(&path) {
        Ok(Vec::new())
    } else if path.is_dir() {
        path.read_dir()?
            .map(|d|
                scan_dir(d.map_err(|_| Error::Other("".to_string()))?.path(), ignored))
            .collect::<Result<Vec<Vec<PathBuf>>, Error>>()
            .map(|v|
                v.into_iter()
                    .flatten()
                    .collect())
    } else {
        Ok(Vec::from([path]))
    }.map(|v| v.into_iter().filter(|p| p.is_file()).collect())
}

pub fn init_logger() -> Result<(), SetLoggerError> {
    let dir = Path::new("./logs/");
    if !dir.exists() {
        fs::create_dir(dir).expect("Could not create log dir");
    }

    let log_filename = format!("./logs/{}.log", Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)).replace(":", "_");

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
            File::create(&log_filename).unwrap_or_else(|_| { panic!("{}", log_filename) }),
        ),
    ])
}

pub fn convert_hash_algorithm(name: &str) -> Option<&'static Algorithm> {
    match name.to_lowercase().as_str() {
        "sha256" => Some(&SHA256),
        "sha384" => Some(&SHA384),
        "sha512" => Some(&SHA512),
        "sha512_256" => Some(&SHA512_256),
        _ => None,
    }
}