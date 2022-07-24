use std::cmp::min;
use std::collections::BTreeSet;
use std::fs;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use base32::Alphabet;
use futures_util::stream::StreamExt;
use log::{error, info, trace};
use reqwest::{Client, Response};
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Sender;
use url::Url;

use crate::{convert_hash_algorithm, Error, FileInfo, hash_file, Info, scan_dir};

#[derive(Debug)]
/// Messages sent from the update task
pub enum Message {
    /// (id, total_size) Created a new download state
    AddState(u32, u64),
    /// (id, done) Updated download with done bytes downloaded
    UpdateState(u32, u64),

    /// Fetch of files to updated completed
    FetchDone,
    /// Unnecessary files deletion completed
    CleanDone,
    /// Update completed
    DownloadDone,

    /// (err) Download interrupted with err
    Interrupted(Box<Error>),
}

/// State of a download
pub struct DownloadState {
    done: u64,
    total: u64,
}

impl DownloadState {
    /// Creates a DownloadState
    /// # Arguments
    /// * 'total' total bytes to be downloaded
    pub fn new(total: u64) -> Self {
        Self { done: 0, total }
    }

    /// Returns the amount of bytes already downloaded
    pub fn done(&self) -> u64 {
        self.done
    }
    /// Returns the total of bytes already downloaded and to be downloaded
    pub fn total(&self) -> u64 {
        self.total
    }

    /// Updates the amount of bytes downloaded to 'done'
    pub fn set_done(&mut self, done: u64) {
        self.done = min(self.total, done);
    }
}

struct DownloadStateHandle {
    id: u32,
    sender: Sender<Message>,
    state: DownloadState,
}

impl DownloadStateHandle {
    async fn new(id: u32, sender: Sender<Message>, total_size: u64) -> Result<DownloadStateHandle, SendError<Message>> {
        sender.send(Message::AddState(id, total_size)).await?;
        Ok(Self {
            id,
            sender,
            state: DownloadState::new(total_size),
        })
    }

    fn done(&self) -> u64 {
        self.state.done
    }

    async fn set_done(&mut self, done: u64) -> Result<(), SendError<Message>> {
        self.state.set_done(done);
        self.sender.send(Message::UpdateState(self.id, done)).await
    }
}

/// Starts the update of the current directory
/// # Argument
/// * 'sender' - A mpsc sender to transmit messages about the update
pub async fn update(sender: Sender<Message>, fetch_url: String, download_dir: String) {
    async fn try_update(sender: Sender<Message>, fetch_url: &str, download_dir: &str) -> Result<(), Error> {
        info!("Fetching json from {}", fetch_url);

        let client = Client::new();
        let info: Info = client.get(fetch_url).send().await?.json::<Info>().await?;
        let mut requests = Vec::new();
        let mut i = 0;
        let dl_dir = Path::new(download_dir).to_path_buf();
        let required_files: BTreeSet<PathBuf> = info.files.iter().map(|fi| dl_dir.join(&fi.path)).collect();

        for file_info in &info.files {
            if let Some(request) = generate_download_request(&client, &info, file_info, i, sender.clone(), &dl_dir).await? {
                trace!("Registering update request for {}", file_info.path);
                requests.push(request);
                i += 1;
            }
        }
        sender.send(Message::FetchDone).await?;

        info!("Cleaning directory");
        for file in scan_dir(dl_dir)? {
            if !required_files.contains(&file) {
                trace!("Removing file {}", file.display());
                fs::remove_file(file)?;
            }
        }
        info!("Cleaning completed");
        sender.send(Message::CleanDone).await?;

        info!("Starting downloads");
        for req in requests {
            trace!("Downloading {}", req.2.display());
            download_file(req.0, req.1, req.2).await?;
        }
        info!("Downloading complete");
        sender.send(Message::DownloadDone).await?;

        Ok(())
    }

    if let Err(e) = try_update(sender.clone(), &fetch_url, &download_dir).await {
        error!("Download interrupted by {:#?}", e);
        sender.send(Message::Interrupted(Box::new(e))).await.unwrap();
    }
}

async fn generate_download_request<'r>(client: &Client, info: &Info, file_info: &FileInfo, id: u32, sender: Sender<Message>, dl_dir: &PathBuf) -> Result<Option<(Response, DownloadStateHandle, PathBuf)>, Error> {
    let url = Url::parse(info.base_url.as_str())?.join(&file_info.path)?;
    let path = dl_dir.join(&file_info.path);

    if !path.exists() || {
        let local_hash = hash_file(&path, convert_hash_algorithm(info.algorithm.as_str()).expect(format!("Unknown algorithm: {}", info.algorithm).as_str()))?;
        let remote_hash = base32::decode(Alphabet::Crockford, file_info.hash.as_str()).ok_or_else(|| Error::Other("Could not parse hash".to_string()))?;

        *local_hash.as_ref() != *remote_hash
    } {
        let res = client
            .get(url)
            .send()
            .await?;
        let total_size = res
            .content_length()
            .ok_or_else(|| "Failed to get content length".to_string())?;

        let state = DownloadStateHandle::new(id, sender, total_size).await?;
        Ok(Some((res, state, path)))
    } else {
        Ok(None)
    }
}

async fn download_file(res: Response, mut state: DownloadStateHandle, path: PathBuf) -> Result<(), Error>
{
    create_dir_all(path.parent().unwrap())?;
    let mut file = File::create(&path).map_err(|_| format!("Failed to create file '{}'", &path.to_str().unwrap()))?;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;

        let new = state.done() + (chunk.len() as u64);
        state.set_done(new).await?;
    }

    Ok(())
}