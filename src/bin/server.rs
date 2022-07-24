#[macro_use]
extern crate rocket;

use std::path::{Path, PathBuf};

use rocket::fs::{NamedFile, relative};
use rocket::serde::json::Json;

use bootstrap::{convert_hash_algorithm, Error, FileInfo, hash_file, Info, scan_dir};

pub fn hash_algorithm() -> &'static str {
    option_env!("HASH_ALGORITHM").unwrap_or("sha256")
}

pub fn generate_info(path_base: &str) -> Result<Info, Error> {
    let ignored = vec![PathBuf::from("empty"), PathBuf::from("ignored/")];
    Ok(Info {
        base_url: "http://localhost:8000/files/".to_string(),
        algorithm: hash_algorithm().to_string(),
        files: scan_dir(PathBuf::from(path_base), &ignored.iter().map(|p| PathBuf::from(path_base).join(p)).collect())?
            .into_iter()
            .map(|file_path|
                Ok(FileInfo {
                    path: String::from(file_path.clone().strip_prefix(path_base)?.to_str().ok_or(Error::Other("Could not compute file path".to_string()))?),
                    hash: base32::encode(base32::Alphabet::Crockford, hash_file(&file_path, convert_hash_algorithm(hash_algorithm()).expect(format!("Unknown algorithm: {}", hash_algorithm()).as_str()))?.as_ref()),
                }))
            .collect::<Result<Vec<FileInfo>, Error>>()?,
        ignored_files: ignored,
    })
}

#[get("/")]
pub async fn info() -> Result<Json<Info>, Json<()>> {
    generate_info("./static").map(Json::from).map_err(|_| Json(()))
}

#[get("/<path..>")]
pub async fn files(path: PathBuf) -> Option<NamedFile> {
    let path = Path::new(relative!("static")).join(path);
    NamedFile::open(path).await.ok()
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![info])
        .mount("/files", routes![files])
}