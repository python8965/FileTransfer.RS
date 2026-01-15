#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

extern crate core;

use crate::app::MyApp;

use tracing::{debug, info};
use simple_logger::SimpleLogger;
use std::fs;
use crate::io::{DOWNLOAD_PATH, UPLOAD_PATH};

mod app;
mod io;
mod network;
mod tool;

fn init() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        debug!(
            "DOWNLOAD PATH IS EXIST?: {:?}",
            DOWNLOAD_PATH.try_exists()
        );

        if !DOWNLOAD_PATH.try_exists().unwrap() {
            debug!("PATH : {}", DOWNLOAD_PATH.to_str().unwrap());
            fs::create_dir_all(&*DOWNLOAD_PATH).unwrap();
        }

        if !UPLOAD_PATH.try_exists().unwrap() {
            fs::create_dir_all(&*UPLOAD_PATH).unwrap();
        }
    }
}
// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {


    SimpleLogger::new().init().unwrap();

    info!("App Started");
    init();
    // Log to stdout (if you run with `RUST_LOG=debug`).
    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|_cc| Ok(Box::new(MyApp::new()))),
    )
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();
    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(MyApp::new()))),
            )
            .await
            .expect("failed to start eframe");
    });
}
