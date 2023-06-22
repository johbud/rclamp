use crate::helpers;
use crate::helpers::EXPLORER;
use crate::helpers::FINDER;
use open;
use std::ffi::OsString;
use std::fs;
use std::io;

use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Project {
    pub name: String,
    pub name_sanitized: String,
    pub path: PathBuf,
    pub pipeline_dir: PathBuf,
    pub pipeline_dir_name: String,
    pub work_dir: PathBuf,
    pub work_dir_name: String,
    pub dailies_dir: PathBuf,
    pub dailies_dir_name: String,
    pub deliveries_dir: PathBuf,
    pub deliveries_dir_name: String,
    pub extra_dir_names: Vec<String>,
    pub work_sub_dirs: Vec<String>,
}

impl Project {
    /// Finds projects matching the template project in the specified directory.
    pub fn find_projects(
        projects_dir: PathBuf,
        template_project: Project,
    ) -> Result<Vec<Project>, io::Error> {
        let mut projects: Vec<Project> = Vec::new();

        let dir_listing = match fs::read_dir(projects_dir) {
            Ok(listing) => listing,
            Err(error) => return Err(error),
        };

        for result in dir_listing {
            let item = match result {
                Ok(i) => i,
                Err(_e) => continue,
            };

            let sub_listing = match fs::read_dir(item.path()) {
                Ok(i) => i,
                Err(_e) => continue,
            };

            for sub_result in sub_listing {
                let sub_entry = match sub_result {
                    Ok(i) => i,
                    Err(_e) => continue,
                };

                let tp = template_project.clone();

                if sub_entry.path().is_dir() {
                    if sub_entry.file_name() == OsString::from(tp.pipeline_dir_name.clone()) {
                        let name = match item.file_name().into_string() {
                            Ok(n) => n,
                            Err(_n) => String::from("Error parsing filename"),
                        };
                        projects.push(Project::new(
                            name,
                            tp.path,
                            tp.pipeline_dir_name,
                            tp.work_dir_name,
                            tp.dailies_dir_name,
                            tp.deliveries_dir_name,
                            tp.extra_dir_names,
                            tp.work_sub_dirs,
                        ));
                    }
                }
            }
        }
        projects.sort();
        Ok(projects)
    }

    /// Create an actual project folder with subfolder.
    pub fn create(&self) -> Result<(), io::Error> {
        let mut subfolders: Vec<PathBuf> = Vec::new();
        subfolders.push(self.pipeline_dir.clone());
        subfolders.push(self.dailies_dir.clone());
        subfolders.push(self.deliveries_dir.clone());
        subfolders.push(self.work_dir.clone());
        for dir in self.extra_dir_names.clone() {
            subfolders.push(PathBuf::from(dir));
        }

        match fs::create_dir(&self.path) {
            Ok(()) => (),
            Err(e) => return Err(e),
        }

        for f in subfolders {
            let mut p = self.path.clone();
            p.push(f);
            match fs::create_dir(p) {
                Ok(()) => (),
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    /// Get a new project struct, does not create a project folder.
    pub fn new(
        name: String,
        projects_dir: PathBuf,
        pipeline_dir_name: String,
        work_dir_name: String,
        dailies_dir_name: String,
        deliveries_dir_name: String,
        extra_dir_names: Vec<String>,
        work_sub_dirs: Vec<String>,
    ) -> Self {
        let mut p: PathBuf = projects_dir.clone();
        p.push(PathBuf::from(&name));

        let mut work_path: PathBuf = p.clone();
        work_path.push(PathBuf::from(&work_dir_name));

        let mut dailies_path: PathBuf = p.clone();
        dailies_path.push(PathBuf::from(&dailies_dir_name));

        let mut deliveries_path: PathBuf = p.clone();
        deliveries_path.push(PathBuf::from(&deliveries_dir_name));

        let mut pipe_path: PathBuf = p.clone();
        pipe_path.push(PathBuf::from(&pipeline_dir_name));

        Self {
            name: name.clone(),
            name_sanitized: helpers::sanitize_string(name.clone()),
            path: p,
            pipeline_dir_name: pipeline_dir_name,
            pipeline_dir: pipe_path,
            work_dir_name: work_dir_name,
            work_dir: work_path,
            dailies_dir_name: dailies_dir_name,
            dailies_dir: dailies_path,
            deliveries_dir: deliveries_path,
            deliveries_dir_name: deliveries_dir_name,
            extra_dir_names: extra_dir_names,
            work_sub_dirs: work_sub_dirs,
        }
    }

    pub fn open_dailies_folder(&self) {
        let path = OsString::from(&self.dailies_dir);

        let command = if cfg!(windows) { EXPLORER } else { FINDER };

        match open::with(path, command) {
            Ok(()) => (),
            Err(_e) => (),
        }
    }

    pub fn open_deliveries_folder(&self) {
        let path = OsString::from(&self.deliveries_dir);
        let command = if cfg!(windows) { EXPLORER } else { FINDER };

        match open::with(path, command) {
            Ok(()) => (),
            Err(_e) => (),
        }
    }
}
