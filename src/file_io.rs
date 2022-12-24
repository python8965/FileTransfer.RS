use egui::{self, trace, Ui};
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use std::{fs, io};
use log::debug;

pub static ROOT_PATH: Lazy<&Path> = Lazy::new(|| Path::new(".\\root\\"));
pub static DOWNLOAD_PATH: Lazy<&Path> = Lazy::new(|| Path::new(".\\root\\downloads\\"));
pub static UPLOAD_PATH: Lazy<&Path> = Lazy::new(|| Path::new(".\\root\\uploads\\"));

#[derive(Default, Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: usize,
}

impl FileInfo {
    fn new(path: PathBuf, size: usize) -> Self {
        FileInfo { path, size }
    }

    pub fn name(&self) -> &str {
        self.path.file_name().unwrap().to_str().unwrap()
    }
}

#[derive(Default, Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FolderInfo {
    path: PathBuf,
    size: usize,
    elements: Vec<FileSystemElement>,
}

impl FolderInfo {
    fn new(_path: impl AsRef<Path>, size: usize, elements: Vec<FileSystemElement>) -> Self {
        FolderInfo {
            size,
            path: _path.as_ref().to_path_buf(),
            elements,
        }
    }

    pub fn name(&self) -> &str {
        self.path.file_name().unwrap().to_str().unwrap()
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
enum FileSystemElement {
    Folder(Box<FolderInfo>, bool),
    File(FileInfo, bool),
}

#[derive(Default, Debug)]
struct RootFolder{
    inner: FolderInfo,
    selected : bool
}

impl RootFolder{
    fn new(root: FolderInfo) -> Self{
        Self{
            inner: root,
            selected: false
        }
    }
}

#[derive(Default, Debug)]
pub struct FileSystem {
    root_folder: RootFolder,
    selected_files: Vec<FileInfo>,
}

impl FileSystem {
    pub fn new() -> Self {
        let root_folder = scan_root_folder().unwrap();

        FileSystem {
            root_folder: RootFolder::new(root_folder),
            selected_files: vec![],
        }
    }

    pub fn selected_files(&mut self) -> &Vec<FileInfo> {
        &self.selected_files
    }

    pub fn file_system_ui(&mut self, ui: &mut Ui) {
        folder_ui(ui, &mut self.root_folder.inner,&mut self.root_folder.selected, &mut self.selected_files);
        ui.separator();
        for file in self.selected_files.iter() {
            ui.label(format!("Selected {}", file.name()));
        }
    }


}

pub fn build_ui(ui: &mut Ui, folder: &mut FolderInfo, selected_files: &mut Vec<FileInfo>){
    for file_system_element in &mut folder.elements {
        match file_system_element {
            FileSystemElement::Folder(folder,selected) => {
                folder_ui(ui, folder, selected, selected_files);
            }
            FileSystemElement::File(file, selected) => {
                let label = ui.toggle_value(&mut *selected, file.name());

                if label.clicked() {
                    if *selected {
                        selected_files.push(file.clone());
                    } else {
                        selected_files.remove(
                            selected_files
                                .iter()
                                .position(|x| x.path == file.path)
                                .unwrap(),
                        );
                    }
                }
            }
        }
    }
}

fn folder_ui(ui: &mut Ui, folder: &mut FolderInfo, _selected: &mut bool, selected_files: &mut Vec<FileInfo> ){
        ui.collapsing(format!("{} Folder", folder.name()), |ui| {
                ui.horizontal(|ui|{if ui.checkbox( _selected, "").changed() {
                    selected(folder, _selected, selected_files);
                }
                    
                ui.label("Is Selected");
            });


            build_ui(ui, folder, selected_files);
        });
}

fn selected(folder: &mut FolderInfo, is_selected: &mut bool, selected_files: &mut Vec<FileInfo> ){
    for mut file_system_element in &mut folder.elements {
        match file_system_element {
            FileSystemElement::Folder(folder, _selected) => {
                debug!("{:?} {:?}", is_selected, _selected);
                if *_selected != *is_selected {
                    *_selected = *is_selected;

                    selected(&mut *folder, is_selected, selected_files);
                }
            }
            FileSystemElement::File(file, _selected) => {
                if *_selected != *is_selected {
                    *_selected = !*_selected;

                    if *is_selected {
                        selected_files.push(file.clone());
                    } else {
                        selected_files.remove(
                            selected_files
                                .iter()
                                .position(|x| x.path == file.path)
                                .unwrap(),
                        );
                    }
                }
            }
        }
    }
}

fn scan_root_folder() -> io::Result<FolderInfo> {
    fn scan(dir: fs::ReadDir, data: &mut Vec<FileSystemElement>) -> io::Result<()> {
        for res in dir {
            let dir_entry = res.expect("err in direntry");

            let path = dir_entry.path();

            match dir_entry.metadata()? {
                meta if meta.is_dir() => {
                    let mut folder = FolderInfo::new(dir_entry.path(), meta.len() as usize, vec![]);

                    scan(fs::read_dir(path)?, &mut folder.elements)?;
                    data.push(FileSystemElement::Folder(Box::from(folder), false));
                }
                meta if meta.is_file() => {
                    data.push(FileSystemElement::File(
                        FileInfo::new(dir_entry.path(), meta.len() as usize),
                        false,
                    ));
                }
                _ => {}
            }
        }

        Ok(())
    }
    let mut root = FolderInfo::new(*UPLOAD_PATH, dir_size(*UPLOAD_PATH)? as usize, vec![]);
    scan(fs::read_dir(*UPLOAD_PATH)?, &mut root.elements)?;
    Ok(root)
}

fn dir_size(path: impl Into<PathBuf>) -> io::Result<u64> {
    fn dir_size(mut dir: fs::ReadDir) -> io::Result<u64> {
        dir.try_fold(0, |acc, file| {
            let file = file?;
            let size = match file.metadata()? {
                data if data.is_dir() => dir_size(fs::read_dir(file.path())?)?,
                data => data.len(),
            };
            Ok(acc + size)
        })
    }

    dir_size(fs::read_dir(path.into())?)
}
