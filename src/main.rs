use relm4::RelmApp;
mod app;
use crate::app::{App, config::info::APP_ID};

fn main() {
    let app = RelmApp::new(APP_ID);
    app.run::<App>(());
}
