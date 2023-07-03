use log::info;

use crate::helpers::EXPLORER;
use crate::helpers::FINDER;
use crate::File;
use crate::Project;

use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs::{self, DirEntry};
use std::io;
use std::path::PathBuf;

/// Can include additional metadata for task directories. Currently only informs whether a dir is a task or not.
#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct TaskNodeMetadata {
    pub is_task: bool,
    pub work_dir_name: String,
}

/// Represents a directory.
#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct TaskTreeNode {
    pub name: String,
    pub path: PathBuf,
    pub metadata: TaskNodeMetadata,
    pub children: Vec<TaskTreeNode>,
}

impl TaskTreeNode {
    /// Returns a new representation of a task directory, from a given path.
    pub fn from_path(path: PathBuf) -> Result<TaskTreeNode, io::Error> {
        let name = String::from(
            path.file_name()
                .unwrap_or(OsStr::new(""))
                .to_str()
                .unwrap_or(""),
        );

        let mut node = TaskTreeNode::new(name.clone(), path.clone());
        
        let mut check_for_work = path.clone();
        let mut check_for_output = path.clone();
        
        check_for_work.push("01_work");
        check_for_output.push("02_output");
        
        if check_for_work.is_dir() && check_for_output.is_dir() {
            node.metadata.is_task = true;
            info!("Found task: {} at {}", &name, &path.display());
            return Ok(node);
        }

        let dir_listing = match fs::read_dir(&path) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };

        info!("Found folder: {} at {}", &name, &path.display());
        for result in dir_listing {
            let item: DirEntry = match result {
                Ok(r) => r,
                Err(_e) => continue,
            };

            if item.path().is_file() {
                continue;
            }

            let child = match TaskTreeNode::from_path(item.path()) {
                Ok(c) => c,
                Err(e) => return Err(e),
            };

            node.children.push(child);
        }

        Ok(node)
    }

    /// Returns a new representation of a task directory.
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            name: name,
            path: path,
            metadata: TaskNodeMetadata { is_task: false,
            work_dir_name: String::from("01_work") },
            children: Vec::new(),
        }
    }

    /// Opens the tasks output directory in Explorer or Finder.
    pub fn open_output(&self) {
        let mut output_path: PathBuf = self.path.clone();
        output_path.push(PathBuf::from("02_output"));
        let path = OsString::from(output_path);

        let command = if cfg!(windows) { EXPLORER } else { FINDER };

        match open::with(path, command) {
            Ok(()) => (),
            Err(_e) => (),
        }
    }

    pub fn get_work_path(&self) -> PathBuf {
        let mut path = self.path.clone();
        path.push(PathBuf::from(&self.metadata.work_dir_name));
        path
    }

    /// Create a task folder and subfolders on drive. Remember to refresh task tree in ui.
    pub fn create_task(&self, name: String, project: Project) -> Result<(), io::Error> {
        let mut task_path = self.path.clone();
        task_path.push(PathBuf::from(name));

        match fs::create_dir(&task_path) {
            Ok(()) => (),
            Err(e) => return Err(e),
        };

        for d in project.work_sub_dirs {
            let mut dir = task_path.clone();
            dir.push(PathBuf::from(d));

            match fs::create_dir(dir) {
                Ok(()) => (),
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Create a folder on drive. Remember to refresh task tree in ui.
    pub fn create_folder(&self, name: String) -> Result<(), io::Error> {
        let mut folder_path = self.path.clone();
        folder_path.push(PathBuf::from(name));

        match fs::create_dir(&folder_path) {
            Ok(()) => (),
            Err(e) => return Err(e),
        };
        Ok(())
    }

    /// Returns a list of workfiles in the tasks work-folder.
    pub fn find_workfiles(&self, work_dir_name: String) -> Result<Vec<File>, io::Error> {
        let mut work_dir = self.path.clone();
        let mut files = Vec::new();
        work_dir.push(PathBuf::from(work_dir_name));

        let dir_listing = match fs::read_dir(work_dir) {
            Ok(d) => d,
            Err(e) => return Err(e),
        };

        for i in dir_listing {
            let item = match i {
                Ok(f) => f,
                Err(_e) => continue,
            };

            if item.path().is_dir() {
                continue;
            }

            match File::from_path(item.path()) {
                Ok(f) => files.push(f),
                Err(_e) => continue,
            };
        }

        Ok(files)
    }
}
