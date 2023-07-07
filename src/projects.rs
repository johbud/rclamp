use crate::helpers;
use crate::helpers::EXPLORER;
use crate::helpers::FINDER;
use crate::helpers::PROJECT_FILE_NAME;
use log::{error, info};
use open;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Project {
    pub name: String,
    pub name_sanitized: String,
    pub pipeline_dir_name: String,
    pub work_dir_name: String,
    pub dailies_dir_name: String,
    pub deliveries_dir_name: String,
    pub extra_dir_names: Vec<String>,
    pub work_sub_dirs: Vec<String>,
}

impl Project {
    pub fn get_path(&self, projects_dir: &PathBuf) -> PathBuf {
        let mut path = projects_dir.clone();
        path.push(PathBuf::from(&self.name_sanitized));
        path
    }

    pub fn get_work_path(&self, projects_dir: &PathBuf) -> PathBuf {
        let mut work_path = self.get_path(projects_dir);
        work_path.push(PathBuf::from(&self.work_dir_name));
        work_path
    }

    pub fn get_dailies_path(&self, projects_dir: &PathBuf) -> PathBuf {
        let mut dailies_path = self.get_path(projects_dir);
        dailies_path.push(PathBuf::from(&self.dailies_dir_name));
        dailies_path
    }

    pub fn get_deliveries_path(&self, projects_dir: &PathBuf) -> PathBuf {
        let mut deliveries_path = self.get_path(projects_dir);
        deliveries_path.push(PathBuf::from(&self.deliveries_dir_name));
        deliveries_path
    }

    pub fn get_pipeline_path(&self, projects_dir: &PathBuf) -> PathBuf {
        let mut pipeline_path = self.get_path(projects_dir);
        pipeline_path.push(PathBuf::from(&self.pipeline_dir_name));
        pipeline_path
    }

    /// Finds projects matching the template project in the specified directory.
    pub fn find_projects(
        projects_dir: PathBuf,
        _template_project: Project,
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

            let mut project_config_path = item.path().clone();
            project_config_path.push(PathBuf::from(PROJECT_FILE_NAME));

            if project_config_path.exists() {
                let project = match Project::read_project(project_config_path) {
                    Ok(p) => p,
                    Err(_e) => continue,
                };
                projects.push(project);
            }
        }
        projects.sort();
        info!("Found projects: {:?}", projects);
        Ok(projects)
    }

    fn read_project(path: PathBuf) -> Result<Project, io::Error> {
        info!("Attempting to open project: {}", path.display());
        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) => {
                error!("Could not open project: {}", e);
                return Err(e);
            }
        };
        let project: Project = match serde_yaml::from_reader(file) {
            Ok(p) => p,
            Err(e) => {
                error!("Could not open project: {}", e);
                return Err(io::Error::new(io::ErrorKind::Other, e.to_string()));
            }
        };
        Ok(project)
    }

    /// Create an actual project folder with subfolder.
    pub fn create(&self, projects_dir: PathBuf) -> Result<(), io::Error> {
        let mut subfolders: Vec<PathBuf> = Vec::new();
        subfolders.push(self.get_dailies_path(&projects_dir));
        subfolders.push(self.get_deliveries_path(&projects_dir));
        subfolders.push(self.get_work_path(&projects_dir));
        for dir in self.extra_dir_names.clone() {
            subfolders.push(PathBuf::from(dir));
        }

        match fs::create_dir(self.get_path(&projects_dir)) {
            Ok(()) => (),
            Err(e) => return Err(e),
        }

        for f in subfolders {
            let mut p = self.get_path(&projects_dir);
            p.push(f);
            match fs::create_dir(p) {
                Ok(()) => (),
                Err(e) => return Err(e),
            }
        }

        let mut file_path = self.get_path(&projects_dir);
        file_path.push(PathBuf::from(PROJECT_FILE_NAME));

        let file = match std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_path)
        {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to open file for writing: {}", e);
                return Err(e);
            }
        };

        match serde_yaml::to_writer(file, self) {
            Ok(()) => (),
            Err(e) => {
                error!("Failed to write project file: {}", e);
                return Err(io::Error::new(io::ErrorKind::Other, e.to_string()));
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
            pipeline_dir_name,
            work_dir_name,
            dailies_dir_name,
            deliveries_dir_name,
            extra_dir_names,
            work_sub_dirs,
        }
    }

    pub fn open_dailies_folder(&self, projects_dir: PathBuf) {
        let path = OsString::from(self.get_dailies_path(&projects_dir));

        let command = if cfg!(windows) { EXPLORER } else { FINDER };

        match open::with(path, command) {
            Ok(()) => (),
            Err(_e) => (),
        }
    }

    pub fn open_deliveries_folder(&self, projects_dir: PathBuf) {
        let path = OsString::from(self.get_deliveries_path(&projects_dir));
        let command = if cfg!(windows) { EXPLORER } else { FINDER };

        match open::with(path, command) {
            Ok(()) => (),
            Err(_e) => (),
        }
    }
}
