use std::{
    cell::RefCell,
    collections::HashMap,
    fs::{read_dir, File as FsFile},
    io::{BufReader, Read},
    rc::Rc,
};

use super::{CodeDirRef, CodeDirectory, CodeFile, CodeFileRef, CodeRepoObject, CodeRepoObjectType};

pub struct Directory {
    path: String,
    name: String,
    initialized: RefCell<bool>,
    dirs: RefCell<HashMap<String, CodeDirRef>>,
    files: RefCell<HashMap<String, CodeFileRef>>,
}

impl Directory {
    pub fn new(path: &str, name: &str) -> Self {
        Self {
            path: path.to_string(),
            name: name.to_string(),
            initialized: RefCell::new(false),
            dirs: RefCell::new(HashMap::new()),
            files: RefCell::new(HashMap::new()),
        }
    }

    fn initialize(&self) {
        *self.initialized.borrow_mut() = true;

        let full_path = self.path.clone() + "/" + &self.name;

        let entries = read_dir(full_path.clone());
        if entries.is_err() {
            return;
        }

        for entry in entries.unwrap() {
            if let Ok(dir_entry) = entry {
                let name = dir_entry.file_name().into_string();
                if name.is_err() {
                    continue;
                }
                let name = name.unwrap();

                match dir_entry.file_type() {
                    Ok(file_type) => {
                        if file_type.is_dir() {
                            let dir = Rc::new(RefCell::new(Directory::new(&full_path, &name)));
                            self.dirs.borrow_mut().insert(name, dir);
                        } else if file_type.is_file() {
                            let file = Rc::new(RefCell::new(File::new(&full_path, &name)));
                            self.files.borrow_mut().insert(name, file);
                        }
                    }
                    Err(_) => {}
                }
            }
        }
    }
}

impl CodeRepoObject for Directory {
    fn get_type(&self) -> CodeRepoObjectType {
        CodeRepoObjectType::Directory
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CodeDirectory for Directory {
    fn get_dir(&self, name: &str) -> Option<CodeDirRef> {
        if !self.initialized.borrow().clone() {
            self.initialize();
        }

        let dirs = self.dirs.borrow();
        let dir = dirs.get(name);
        match dir {
            Some(dir) => Some(dir.clone()),
            None => None,
        }
    }

    fn get_file(&self, name: &str) -> Option<CodeFileRef> {
        if !self.initialized.borrow().clone() {
            self.initialize();
        }

        let file_name = name.to_string() + ".boli";

        let files = self.files.borrow();
        let file = files.get(&file_name);
        match file {
            Some(file) => Some(file.clone()),
            None => None,
        }
    }
}

pub struct File {
    path: String,
    name: String,
}

impl File {
    pub fn new(path: &str, name: &str) -> Self {
        Self {
            path: path.to_string(),
            name: name.to_string(),
        }
    }
}

impl CodeRepoObject for File {
    fn get_type(&self) -> CodeRepoObjectType {
        CodeRepoObjectType::File
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl CodeFile for File {
    fn read(&self) -> String {
        let mut buffer = String::new();
        let file_path = self.path.clone() + "/" + &self.name;
        match FsFile::open(&file_path) {
            Ok(file) => {
                let mut reader = BufReader::new(file);
                match reader.read_to_string(&mut buffer) {
                    Ok(_) => buffer,
                    Err(_) => "".to_string(),
                }
            }
            Err(_) => "".to_string(),
        }
    }
}
