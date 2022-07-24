use std::collections::HashMap;
use std::process::Command;

use clap::Parser;
use fltk::{app, prelude::*, window::Window};
use fltk::misc::Progress;
use tokio::sync::mpsc::error;

use bootstrap::init_logger;
use bootstrap::updater::{DownloadState, Message, update};

#[derive(Parser, Debug)]
#[clap(name = "bootstrap",
author = "Ludovic Mermod, <ludovic.mermod@gmail.com>",
version = "1.1",
about = "Updates an application from a remote server")]
struct Cli {
    #[clap(default_value = std::env ! ("FETCH_URL"))]
    fetch: String,

    #[clap(default_value = option_env ! ("DOWNLOAD_DIR").or(Some("./")).unwrap())]
    dir: String,

    #[clap(default_value = std::env ! ("EXECUTABLE_NAME"))]
    executable_name: String,

    #[clap(long, takes_value = false)]
    no_gui: bool,

    #[clap(long, takes_value = false)]
    raw_data: bool,
}

#[tokio::main]
async fn main() {
    init_logger().expect("Unable to init logger");
    let cli = Cli::parse();
    println!("{:?}", cli);

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    let mut map = HashMap::new();
    tokio::spawn(update(tx, cli.fetch, cli.dir));

    if !cli.no_gui {
        let app = app::App::default();
        let mut wind = Window::new(100, 100, 400, 80, "Updating");
        let mut bar = Progress::new(50, 20, 300, 40, "Fetching update");

        bar.set_minimum(0_f64);
        wind.end();
        wind.show();

        while app.wait() {
            app.redraw();

            let result = rx.try_recv();
            match result {
                Ok(msg) => match msg {
                    Message::AddState(id, size) => {
                        map.insert(id, DownloadState::new(size));
                    }
                    Message::UpdateState(id, done) => {
                        map.get_mut(&id).unwrap().set_done(done);
                        let done = map.values().map(DownloadState::done).sum::<u64>();
                        bar.set_value(done as f64);
                    }
                    Message::FetchDone => {
                        bar.set_maximum(map.values().map(DownloadState::total).sum::<u64>() as f64);
                        bar.set_label("Downloading");
                    }
                    Message::CleanDone => {}
                    Message::DownloadDone => break,
                    Message::Interrupted(_) => break,
                },
                Err(error::TryRecvError::Empty) => {
                    app::sleep(1.0 / 30.0);
                }
                _ => {}
            }
        }
    } else {
        loop {
            match rx.try_recv() {
                Ok(msg) => match msg {
                    Message::AddState(id, size) => {
                        map.insert(id, DownloadState::new(size));
                    }
                    Message::UpdateState(id, done) => {
                        map.get_mut(&id).unwrap().set_done(done);
                        println!("{}", map.values().map(DownloadState::done).sum::<u64>());
                    }
                    Message::FetchDone => {
                        println!("{}", map.values().map(DownloadState::total).sum::<u64>())
                    }
                    Message::CleanDone => {}
                    Message::DownloadDone => break,
                    Message::Interrupted(_) => break,
                },
                Err(error::TryRecvError::Empty) => {
                    app::sleep(1.0 / 30.0);
                }
                _ => {}
            }
        }
    }

    Command::new(format!("./{}", cli.executable_name)).spawn().expect("Could not start application");
}