use std::collections::HashMap;

use fltk::{app, prelude::*, window::Window};
use fltk::misc::Progress;
use tokio::sync::mpsc::error;
use bootstrap::init_logger;

use bootstrap::updater::{DownloadState, Message, update};

#[tokio::main]
async fn main() {
    init_logger().expect("Unable to init logger");

    let app = app::App::default();
    let mut wind = Window::new(100, 100, 400, 80, "Updating");

    let mut bar = Progress::new(50, 20, 300, 40, "Fetching update");
    bar.set_minimum(0_f64);

    wind.end();
    wind.show();

    let mut map = HashMap::new();

    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    tokio::spawn(update(tx));

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
}