mod app;
mod config;
mod languages;
mod ocr;
mod providers;

use gtk::glib;
use gtk::prelude::*;

const APP_ID: &str = "com.whyoolw.Translator";

fn main() -> glib::ExitCode {
    // Single instance: launching again while it's open closes it (toggle).
    // The second process forwards `activate` to the primary instance, whose
    // handler sees an existing window and closes it — quitting the app.
    let app = gtk::Application::builder().application_id(APP_ID).build();
    app.connect_activate(|app| {
        if let Some(win) = app.active_window() {
            win.close();
        } else {
            app::build_app(app);
        }
    });
    // Empty argv: GTK must not try to parse app-specific CLI flags.
    app.run_with_args::<&str>(&[])
}
