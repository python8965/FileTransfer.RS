use std::path::PathBuf;

pub struct DB{
    root_path: PathBuf,
}

impl Default for DB{
    fn default() -> Self{
        Self{
            root_path: PathBuf::from("./root/"),
        }
    }

}

impl DB{
    pub fn new(root_path: PathBuf) -> Self{
        Self{
            root_path
        }
    }
}