use crate::{Project, TaskTreeNode};
use log::{error, info};
use std::fs::{self};
use std::io::{Error, ErrorKind};
use std::{ffi::OsStr, io, path::PathBuf};

/// Represents a workfile found on drive.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct File {
    pub name: String,
    pub path: PathBuf,
    pub extension: String,
    pub version: u32,
}

impl File {
    /// Returns the version number in a presentable format: v###.
    pub fn fmt_version(&self) -> String {
        format!("v{:03}", self.version)
    }

    /// Create a new representation of a workfile, from an existing file path.
    pub fn from_path(path: PathBuf) -> Result<Self, String> {
        let extension = String::from(
            path.extension()
                .unwrap_or(OsStr::new(""))
                .to_str()
                .unwrap_or(""),
        );
        let name = String::from(
            path.file_stem()
                .unwrap_or(OsStr::new(""))
                .to_str()
                .unwrap_or(""),
        );
        let mut version_string = name.clone();
        let version_offset = name.len() - 5;
        let name = version_string.drain(..version_offset).collect();

        if !(&version_string.chars().nth(0).unwrap_or('0') == &'_'
            && &version_string.chars().nth(1).unwrap_or('0') == &'v')
        {
            return Err(String::from("Not a valid filename."));
        }
        version_string.remove(0);
        version_string.remove(0);
        let version: u32 = version_string.parse().unwrap_or(1);
        Ok(Self {
            name: name,
            path: path,
            version: version,
            extension: extension,
        })
    }

    /// Open the file using system default application.
    pub fn open(&self) -> Result<(), io::Error> {
        match open::that(&self.path) {
            Ok(()) => (),
            Err(e) => return Err(e),
        }
        Ok(())
    }

    pub fn create_file(
        name: String,
        task: TaskTreeNode,
        project: Project,
        dcc: Dcc,
    ) -> Result<(), io::Error> {
        let filename = Self::make_filename(&name, &task, &project, &dcc);
        let path = Self::make_path(task, filename);

        match Self::copy_file(path, dcc) {
            Ok(()) => (),
            Err(e) => return Err(e),
        }
        Ok(())
    }

    fn make_filename(name: &String, task: &TaskTreeNode, project: &Project, dcc: &Dcc) -> String {
        let filename = String::from(format!(
            "{}_{}_{}_v001{}",
            project.name_sanitized, task.name, name, dcc.extension
        ));
        filename
    }

    fn make_path(task: TaskTreeNode, name: String) -> PathBuf {
        let mut path = task.get_work_path();
        path.push(PathBuf::from(name));
        path
    }

    fn copy_file(path: PathBuf, dcc: Dcc) -> Result<(), io::Error> {
        match path.try_exists() {
            Ok(b) => {
                if b {
                    return Err(Error::new(ErrorKind::Other, "File already exists!"));
                }
            }
            Err(e) => return Err(e),
        }

        match dcc.template_path.try_exists() {
            Ok(b) => {
                if !b {
                    return Err(Error::new(ErrorKind::Other, "Template file not found."));
                }
            }
            Err(e) => return Err(e),
        }

        match fs::copy(&dcc.template_path, &path) {
            Ok(_u) => return Ok(()),
            Err(e) => {
                error!(
                    "Failed to copy {} to {}: {}",
                    dcc.template_path.display(),
                    path.display(),
                    e.to_string()
                );
                return Err(e);
            }
        }
    }
}

/// Contains data needed to create new workfiles for a dcc.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
pub struct Dcc {
    pub name: String,
    pub extension: String,
    pub template_path: PathBuf,
}

impl Dcc {
    /// Search specified directory for config files and templates, return list of Dcc:s.
    pub fn find_dcc(path: &PathBuf) -> Result<Vec<Dcc>, io::Error> {
        let mut dcc: Vec<Dcc> = Vec::new();

        let dir_listing = match fs::read_dir(path) {
            Ok(listing) => listing,
            Err(e) => return Err(e),
        };

        for l in dir_listing {
            let item = match l {
                Ok(d) => d,
                Err(_e) => continue,
            };

            if item.path().is_file() {
                continue;
            }

            let mut app_config = item.path().clone();
            app_config.push(PathBuf::from("app.yaml"));

            info!("Looking for dcc config: {}", app_config.display());
            let file = match std::fs::File::open(app_config) {
                Ok(f) => f,
                Err(e) => {
                    error!("Could not load config: {}", e);
                    continue;
                }
            };

            let mut dcc_config: Dcc = match serde_yaml::from_reader(file) {
                Ok(c) => c,
                Err(e) => {
                    error!("Could not load dcc: {}", e);
                    continue;
                }
            };

            let mut template_path = item.path().clone();
            template_path.push(PathBuf::from(format!("template{}", dcc_config.extension)));
            if !template_path.exists() {
                error!("Template file not found: {}", template_path.display());
                continue;
            }

            dcc_config.template_path = template_path;

            info!("Found dcc config: {}", dcc_config.name);
            dcc.push(dcc_config);
        }

        Ok(dcc)
    }
}
