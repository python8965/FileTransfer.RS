use crate::file_io::FileSystem;
use crate::network::{FileDownloaderUi, FileSenderUi};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MyApp {
    state: State,
}

impl MyApp {
    pub(crate) fn new() -> Self {
        Self {
            state: State::new(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct State {
    #[serde(skip)]
    file_system: FileSystem,

    #[serde(skip)]
    downloader_ui: FileDownloaderUi,

    #[serde(skip)]
    sender_ui: FileSenderUi,
}

impl State {
    fn new() -> Self {
        Self {
            file_system: FileSystem::new(),
            downloader_ui: FileDownloaderUi::new(),
            sender_ui: FileSenderUi::new(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            egui::ScrollArea::new([true, true]).show(ui, |ui: &mut egui::Ui| {
                ui.heading("Host Application");

                ui.separator();

                let state = &mut self.state;

                state.file_system.file_system_ui(ui);
                state.downloader_ui.ui(ui).unwrap();
                state
                    .sender_ui
                    .ui(ui, state.file_system.selected_files().clone())
                    .unwrap();

                ui.separator();
            });
        });
    }
}
