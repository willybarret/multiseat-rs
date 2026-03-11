use relm4::RelmApp;
mod app;
use crate::app::{App, AppInit, config::info::APP_ID};

fn main() {
    let app = RelmApp::new(APP_ID);
    let initialize_styles = Box::new(|| relm4::set_global_css(include_str!("assets/style.css")));
    app.run::<App>(AppInit { initialize_styles });
}
