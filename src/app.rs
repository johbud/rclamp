use egui::Color32;
use log::{error, info};
use std::env;
use std::io;
use std::path::PathBuf;

use crate::helpers::sanitize_string;
use crate::workfiles::Dcc;
use crate::File;
use crate::Project;
use crate::TaskTreeNode;

pub const SPACING: f32 = 5.;
const TEST_PROJECT_PATH_WIN: &str = "D:\\Dropbox (Personal)\\Annat\\Kod\\rclamp\\test_folder";
const TEST_PROJECT_PATH_MAC: &str =
    "/Users/johnbuddee/Dropbox (Personal)/Annat/Kod/rclamp/test_folder";

#[derive(serde::Deserialize, serde::Serialize)]
struct RclampAppConfig {
    dark_mode: bool,
    projects_dir: PathBuf,
    templates_dir: PathBuf,
    template_project: Project,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct RclampConfig {
    projects_dir: String,
    templates_dir: String,
    pipeline_dir_name: String,
    work_dir_name: String,
    dailies_dir_name: String,
    deliveries_dir_name: String,
    extra_dir_names: Vec<String>,
    work_sub_dirs: Vec<String>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Rclamp {
    current_project: Option<Project>,
    current_project_task_tree: Option<TaskTreeNode>,
    current_task: Option<TaskTreeNode>,
    projects: Vec<Project>,
    projects_filtered: Vec<Project>,
    files: Option<Vec<File>>,
    dcc: Vec<Dcc>,
    config: RclampAppConfig,

    warning_message: String,
    show_create_project: bool,
    show_create_task: bool,
    show_create_folder: bool,
    new_project_name: String,
    new_project_message: String,
    new_project_error: String,
    new_task_name: String,
    new_folder_name: String,
    new_task_parent: TaskTreeNode,
    new_folder_parent: TaskTreeNode,
    new_task_message: String,
    new_folder_message: String,
    new_task_error: String,
    new_folder_error: String,
    new_file_name: String,
    new_file_error: String,
    new_file_type: Dcc,
    project_filter: String,
}

impl Default for Rclamp {
    fn default() -> Self {
        let test_project_path = if cfg!(windows) {
            TEST_PROJECT_PATH_WIN
        } else {
            TEST_PROJECT_PATH_MAC
        };
        let projects_dir = PathBuf::from(test_project_path);
        let mut templates_dir = projects_dir.clone();
        templates_dir.push(PathBuf::from("templates"));
        let template_project = Project::new(
            String::new(),
            projects_dir.clone(),
            String::from("00_pipeline"),
            String::from("02_work"),
            String::from("03_dailies"),
            String::from("04_deliveries"),
            Vec::from([String::from("01_preproduction")]),
            Vec::from([
                String::from("01_work"),
                String::from("02_output"),
                String::from("03_assets"),
            ]),
        );

        let mut warning_message = String::new();
        let mut projects: Vec<Project> = Vec::new();
        match Project::find_projects(projects_dir, template_project.clone()) {
            Ok(p) => projects = p,
            Err(e) => warning_message = String::from(format!("Error finding projects: {}", e)),
        };

        let projects_filtered = projects.clone();

        let mut dcc = Vec::new();
        match Dcc::find_dcc(&templates_dir) {
            Ok(d) => dcc = d,
            Err(e) => warning_message = String::from(format!("Error finding dcc's: {}", e)),
        };
        let empty_task = TaskTreeNode::new(String::new(), PathBuf::new());

        Self {
            current_project: None,
            projects,
            projects_filtered,
            current_project_task_tree: None,
            current_task: None,
            files: None,
            dcc,
            config: RclampAppConfig {
                dark_mode: true,
                projects_dir: PathBuf::from(test_project_path),
                templates_dir,
                template_project,
            },

            warning_message,
            show_create_project: false,
            show_create_task: false,
            show_create_folder: false,
            new_project_name: String::new(),
            new_project_message: String::new(),
            new_project_error: String::new(),
            new_task_name: String::new(),
            new_folder_name: String::new(),
            new_task_parent: empty_task.clone(),
            new_folder_parent: empty_task.clone(),
            new_task_message: String::new(),
            new_folder_message: String::new(),
            new_task_error: String::new(),
            new_folder_error: String::new(),
            new_file_name: String::new(),
            new_file_error: String::new(),
            new_file_type: Dcc {
                name: String::new(),
                extension: String::new(),
                template_path: PathBuf::from("does_not_exist"),
            },
            project_filter: String::new(),
        }
    }
}

impl Rclamp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.

        /*
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        */

        info!("Checking env var for config.");
        let config_path: String = match env::var("RCLAMP_CONFIG") {
            Ok(s) => s,
            Err(e) => {
                error!("Could not load env var: {}", e);
                return Default::default();
            }
        };

        info!("Found config path: {}", config_path);

        let f = match std::fs::File::open(config_path) {
            Ok(f) => f,
            Err(e) => {
                error!("Could not open config file: {}", e);
                return Default::default();
            }
        };

        let config: RclampConfig = match serde_yaml::from_reader(f) {
            Ok(c) => c,
            Err(e) => {
                error!("Could not load config: {}", e);
                return Default::default();
            }
        };
        info!("Read config successfully.");

        let mut rclamp = Rclamp::default();

        let template_project = Project::new(
            String::new(),
            PathBuf::from(&config.projects_dir),
            config.pipeline_dir_name,
            config.work_dir_name,
            config.dailies_dir_name,
            config.deliveries_dir_name,
            config.extra_dir_names,
            config.work_sub_dirs,
        );

        rclamp.config.template_project = template_project;
        rclamp.config.projects_dir = PathBuf::from(config.projects_dir);
        rclamp.config.templates_dir = PathBuf::from(config.templates_dir);

        rclamp
    }

    /// Simply sets the current project.
    fn set_current_project(&mut self, project: Project) {
        self.current_project = Some(project);
    }

    pub fn set_current_task(&mut self, task: TaskTreeNode) {
        let work_subdir = match &self.current_project {
            Some(p) => p
                .work_sub_dirs
                .first()
                .clone()
                .unwrap_or(&String::new())
                .to_owned(),
            None => return,
        };

        self.current_task = Some(task);

        let mut files = match &self.current_task {
            Some(t) => match t.find_workfiles(work_subdir) {
                Ok(v) => v,
                Err(e) => {
                    self.warning_message = String::from(format!("Error opening task: {}", e));
                    return;
                }
            },
            None => return,
        };

        files.sort();
        files.reverse();
        self.files = Some(files);
    }

    fn refresh_all(&mut self, ui: &mut egui::Ui) {
        self.refresh_projects();
        self.refresh_tasks(ui);
        self.refresh_files();
    }

    /// Refreshes the list of projects by calling find_projects.
    fn refresh_projects(&mut self) {
        match Project::find_projects(
            self.config.projects_dir.clone(),
            self.config.template_project.clone(),
        ) {
            Ok(p) => self.projects = p,
            Err(e) => self.warning_message = String::from(format!("Error finding projects: {}", e)),
        }
    }

    /// Refreshes task tree.
    fn refresh_tasks(&mut self, ui: &mut egui::Ui) {
        let project = match &self.current_project {
            Some(p) => p.clone(),
            None => return,
        };
        let tree = match TaskTreeNode::from_path(project.work_dir) {
            Ok(t) => t,
            Err(e) => {
                self.render_task_tree_error(ui, e);
                return;
            }
        };
        self.current_project_task_tree = Some(tree);
    }

    /// Refreshes file list.
    fn refresh_files(&mut self) {
        let task = match &self.current_task {
            Some(t) => t.clone(),
            None => return,
        };
        self.set_current_task(task);
    }

    /// Renders the list of projects.
    fn render_projects(&mut self, ui: &mut egui::Ui) {
        let projects = &self.projects_filtered.clone();

        for p in projects {
            let title = format!("📁 {}", p.name);
            ui.add_space(SPACING);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(title);
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    let open_button = ui.add(egui::Button::new("Open"));
                    let open_deliveries_button = ui.add(egui::Button::new("Deliveries"));
                    let open_dailies_button = ui.add(egui::Button::new("Dailies"));

                    if open_button.clicked() {
                        let _ = &self.open_project(p.clone(), ui);
                    }
                    if open_dailies_button.clicked() {
                        p.open_dailies_folder();
                    }
                    if open_deliveries_button.clicked() {
                        p.open_deliveries_folder();
                    }
                });
            });
            ui.add_space(SPACING);
            ui.add(egui::Separator::default());
        }
    }

    /// First sets the current project, then creates a task tree and assigns it as the current task tree.
    fn open_project(&mut self, project: Project, ui: &mut egui::Ui) {
        self.set_current_project(project.clone());

        let tree = match TaskTreeNode::from_path(project.work_dir) {
            Ok(t) => t,
            Err(e) => {
                self.render_task_tree_error(ui, e);
                return;
            }
        };
        self.current_project_task_tree = Some(tree);
    }

    /// Shows a dialog for creating a task.
    fn create_task_dialog(&mut self, ui: &mut egui::Ui) {
        ui.add_space(SPACING);
        ui.horizontal(|ui| {
            ui.label("Task name: ");
            ui.text_edit_singleline(&mut self.new_task_name);
            let create_task_btn = ui.add(egui::Button::new("Create"));
            let cancel_btn = ui.add(egui::Button::new("❌ Cancel"));

            ui.add_space(SPACING);

            if cancel_btn.clicked() {
                self.show_create_task = false;
                self.new_task_error = String::new();
                self.new_task_message = String::new();
            }

            if create_task_btn.clicked() {
                let project = match &self.current_project {
                    Some(p) => p.clone(),
                    None => {
                        self.new_task_error = String::from("No project open.");
                        return;
                    }
                };

                let task_name = sanitize_string(self.new_task_name.clone());

                if task_name.is_empty() {
                    return;
                }

                match self.new_task_parent.create_task(task_name, project) {
                    Ok(()) => {
                        self.new_task_message = String::from("Successfully created task.");
                        self.new_task_error = String::new();
                    }
                    Err(e) => {
                        self.new_task_error = String::from(format!("Error creating task: {}", e));
                        self.new_task_message = String::new();
                    }
                }
                self.refresh_tasks(ui);
            }
        });
        ui.add_space(SPACING);
    }

    /// Shows a dialog for creating a folder.
    fn create_folder_dialog(&mut self, ui: &mut egui::Ui) {
        ui.add_space(SPACING);
        ui.horizontal(|ui| {
            ui.label("Folder name: ");
            ui.text_edit_singleline(&mut self.new_folder_name);
            let create_folder_btn = ui.add(egui::Button::new("Create"));
            let cancel_btn = ui.add(egui::Button::new("❌ Cancel"));

            ui.add_space(SPACING);

            if cancel_btn.clicked() {
                self.show_create_folder = false;
                self.new_folder_error = String::new();
                self.new_folder_message = String::new();
            }

            if create_folder_btn.clicked() {
                let project = match &self.current_project {
                    Some(p) => p.clone(),
                    None => {
                        self.new_task_error = String::from("No project open.");
                        return;
                    }
                };

                let folder_name = sanitize_string(self.new_folder_name.clone());

                if folder_name.is_empty() {
                    return;
                }

                match self.new_folder_parent.create_task(folder_name, project) {
                    Ok(()) => {
                        self.new_folder_message = String::from("Successfully created folder.");
                        self.new_folder_error = String::new();
                    }
                    Err(e) => {
                        self.new_folder_error =
                            String::from(format!("Error creating folder: {}", e));
                        self.new_folder_message = String::new();
                    }
                }
                self.refresh_tasks(ui);
            }
        });
        ui.add_space(SPACING);
    }

    /// Shows a dialog for creating projects.
    fn create_project_dialog(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.add_space(SPACING);
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.new_project_name);
            let create_project_btn = ui.add(egui::Button::new("Create"));

            ui.add_space(SPACING);

            if create_project_btn.clicked() {
                if self.new_project_name.len() > 0 {
                    match Project::new(
                        self.new_project_name.clone(),
                        self.config.projects_dir.clone(),
                        self.config.template_project.pipeline_dir_name.clone(),
                        self.config.template_project.work_dir_name.clone(),
                        self.config.template_project.dailies_dir_name.clone(),
                        self.config.template_project.deliveries_dir_name.clone(),
                        self.config.template_project.extra_dir_names.clone(),
                        self.config.template_project.work_sub_dirs.clone(),
                    )
                    .create()
                    {
                        Ok(()) => {
                            self.new_project_message =
                                String::from("Successfully created project.");
                            self.new_project_error = String::new();
                        }
                        Err(e) => {
                            self.new_project_error =
                                String::from(format!("Error creating project: {}", e));
                            self.new_project_message = String::new();
                        }
                    }
                    self.refresh_projects();
                }
            }
        });
        ui.add_space(SPACING);
    }

    fn create_file_dialog(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("New workfile name: ");
            ui.text_edit_singleline(&mut self.new_file_name);
            ui.label("File type: ");
            egui::ComboBox::from_id_source("filetype_select")
                .selected_text(format!("{}", self.new_file_type.name))
                .show_ui(ui, |ui| {
                    for d in &self.dcc {
                        ui.selectable_value(&mut self.new_file_type, d.clone(), d.name.clone());
                    }
                });
            let create_file_btn = ui.add(egui::Button::new("Create"));
            if create_file_btn.clicked() {
                if self.current_project.is_none() {
                    return;
                }
                if self.current_task.is_none() {
                    return;
                }

                let file_name = sanitize_string(self.new_file_name.clone());
                if file_name.is_empty() {
                    return;
                }

                match File::create_file(
                    file_name,
                    self.current_task.clone().unwrap(),
                    self.current_project.clone().unwrap(),
                    self.new_file_type.clone(),
                ) {
                    Ok(()) => (),
                    Err(e) => self.warning_message = e.to_string(),
                }
                self.refresh_files();
            }
        });
    }

    /// Top bar containing a few buttons.
    fn render_top_bar(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::menu::bar(ui, |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::RIGHT), |ui| {
                    let text: String;
                    if !self.show_create_project {
                        text = String::from("Create project");
                    } else {
                        text = String::from("❌ Close");
                    }
                    if ui.add(egui::Button::new(text)).clicked() {
                        self.new_project_error = String::new();
                        self.new_project_name = String::new();
                        self.new_project_message = String::new();
                        self.open_or_close_create_project();
                    }
                });
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::RightToLeft),
                    |ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                            ui.label(format!("{}", self.warning_message));
                            ui.label(&self.new_folder_message);
                            ui.label(
                                egui::RichText::new(&self.new_folder_error).color(Color32::RED),
                            );
                            ui.label(&self.new_project_message);
                            ui.label(
                                egui::RichText::new(&self.new_project_error).color(Color32::RED),
                            );
                            ui.label(&self.new_task_message);
                            ui.label(egui::RichText::new(&self.new_task_error).color(Color32::RED));
                        });
                    },
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    let theme_icon = if self.config.dark_mode { "☀" } else { "🌙" };
                    let close_btn = ui.add(egui::Button::new("❌"));
                    let refresh_btn = ui.add(egui::Button::new("🔄"));
                    let theme_btn = ui.add(egui::Button::new(theme_icon));

                    if close_btn.clicked() {
                        frame.close();
                    }
                    if theme_btn.clicked() {
                        self.config.dark_mode = !self.config.dark_mode;
                    }
                    if refresh_btn.clicked() {
                        self.refresh_all(ui);
                    }
                });
            });
        });
    }

    /// Show task tree
    fn render_task_tree(&mut self, ui: &mut egui::Ui) {
        let task = match &self.current_project_task_tree {
            Some(t) => t.clone(),
            None => return,
        };

        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                let new_folder_btn = ui.add(egui::Button::new("+ Folder"));
                let new_task_btn = ui.add(egui::Button::new("+ Task"));
                ui.add_space(SPACING);

                if new_folder_btn.clicked() {
                    self.new_folder_error = String::new();
                    self.new_folder_message = String::new();
                    self.new_folder_name = String::new();
                    self.new_folder_parent = task.clone();
                    self.open_create_folder();
                }
                if new_task_btn.clicked() {
                    self.new_task_error = String::new();
                    self.new_task_message = String::new();
                    self.new_task_name = String::new();
                    self.new_task_parent = task.clone();
                    self.open_create_task();
                }
            });
        });
        for c in &task.children {
            let child = c.clone();
            let _ = &self.tree_child(ui, child);
        }
    }

    fn tree_child(&mut self, ui: &mut egui::Ui, task: TaskTreeNode) {
        if !task.metadata.is_task {
            egui::CollapsingHeader::new(task.name.clone())
                .id_source(task.path.clone())
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                            let new_folder_btn = ui.add(egui::Button::new("+ Folder"));
                            let new_task_btn = ui.add(egui::Button::new("+ Task"));
                            ui.add_space(SPACING);

                            if new_folder_btn.clicked() {
                                self.new_folder_error = String::new();
                                self.new_folder_message = String::new();
                                self.new_folder_name = String::new();
                                self.new_folder_parent = task.clone();
                                self.open_create_folder();
                            }
                            if new_task_btn.clicked() {
                                self.new_task_error = String::new();
                                self.new_task_message = String::new();
                                self.new_task_name = String::new();
                                self.new_task_parent = task.clone();
                                self.open_create_task();
                            }
                        });
                    });
                    for c in &task.children {
                        let child = c.clone();
                        let _ = &self.tree_child(ui, child);
                    }
                    ui.add_space(SPACING);
                });
        } else {
            ui.add_space(SPACING);
            ui.horizontal(|ui| {
                ui.label(&task.name);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    let open_btn = ui.add(egui::Button::new("Open"));
                    let output_btn = ui.add(egui::Button::new("Output"));
                    ui.add_space(SPACING);

                    if open_btn.clicked() {
                        self.set_current_task(task.clone())
                    }
                    if output_btn.clicked() {
                        task.open_output();
                    }
                });
            });
            ui.add_space(SPACING);
        }
    }

    /// If open_project() encounters an error when creating the task tree, this will render the error instead.
    fn render_task_tree_error(&mut self, ui: &mut egui::Ui, error: io::Error) {
        ui.label(error.to_string());
    }

    fn files_table(&mut self, ui: &mut egui::Ui) {
        use egui_extras::{Column, TableBuilder};

        let files = match &self.files {
            Some(v) => v.clone(),
            None => return,
        };

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::initial(250.0))
            .column(Column::initial(75.0))
            .column(Column::remainder())
            .min_scrolled_height(0.0)
            .header(20., |mut header| {
                header.col(|ui| {
                    ui.strong("Name");
                });
                header.col(|ui| {
                    ui.strong("Version");
                });
                header.col(|ui| {
                    ui.strong("Extension");
                });
            })
            .body(|mut body| {
                for f in &files {
                    body.row(20., |mut row| {
                        row.col(|ui| {
                            if ui
                                .add(egui::Label::new(&f.name).sense(egui::Sense::click()))
                                .double_clicked()
                            {
                                match f.open() {
                                    Ok(()) => (),
                                    Err(e) => {
                                        self.warning_message = format!("Error opening file: {}", e)
                                    }
                                }
                            }
                        });
                        row.col(|ui| {
                            ui.label(&f.fmt_version());
                        });
                        row.col(|ui| {
                            ui.label(&f.extension);
                        });
                    })
                }
            });
    }

    fn filter_projects(&mut self, filter_string: String) {
        if filter_string.is_empty() {
            self.projects_filtered = self.projects.clone();
            return;
        }

        let filtered: Vec<Project> = self
            .projects
            .iter()
            .filter(|p| p.name.contains(filter_string.as_str()))
            .cloned()
            .collect();
        self.projects_filtered = filtered;
    }

    fn open_create_folder(&mut self) {
        self.show_create_folder = true;
        self.show_create_project = false;
        self.show_create_task = false;
    }
    fn open_create_task(&mut self) {
        self.show_create_folder = false;
        self.show_create_project = false;
        self.show_create_task = true;
    }
    fn open_or_close_create_project(&mut self) {
        self.show_create_project = !self.show_create_project;
        self.show_create_folder = false;
        self.show_create_task = false;
    }
}

impl eframe::App for Rclamp {
    /// Called each time the UI needs repainting, which may be many times per second.
    ///
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.config.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            ui.add_space(SPACING);
            self.render_top_bar(ui, frame);
            ui.add_space(SPACING);
        });

        if self.show_create_project {
            egui::TopBottomPanel::top("create_project_panel").show(ctx, |ui| {
                self.create_project_dialog(ui, frame);
            });
        }

        egui::SidePanel::left("first_left_panel").show(ctx, |ui| {
            // Left panel
            ui.add_space(SPACING);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                ui.label(format!("Filter"));
                let filter_edit = ui.text_edit_singleline(&mut self.project_filter);
                if filter_edit.changed() {
                    self.filter_projects(self.project_filter.clone());
                }
            });
            ui.add(egui::Separator::default());
            ui.add_space(SPACING);
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.render_projects(ui);
            });
        });

        egui::SidePanel::left("second_left_panel").show(ctx, |ui| {
            // Middle panel
            ui.add_space(SPACING);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                let project_name = match &self.current_project {
                    Some(p) => p.name.clone(),
                    None => String::new(),
                };

                ui.label(format!("Current project: {}", project_name));
            });
            ui.add(egui::Separator::default());
            ui.add_space(SPACING);

            if self.show_create_task {
                ui.add_space(SPACING);
                ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                    self.create_task_dialog(ui);
                });
                ui.add(egui::Separator::default());
                ui.add_space(SPACING);
            }

            if self.show_create_folder {
                ui.add_space(SPACING);
                ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                    self.create_folder_dialog(ui);
                });
                ui.add(egui::Separator::default());
                ui.add_space(SPACING);
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                self.render_task_tree(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Right panel

            let task_name = match &self.current_task {
                Some(t) => t.name.clone(),
                None => String::new(),
            };

            ui.label(format!("Current task: {}", task_name));
            ui.add(egui::Separator::default());
            self.create_file_dialog(ui);
            ui.add(egui::Separator::default());
            ui.add_space(SPACING);

            egui::ScrollArea::vertical().show(ui, |ui| {
                self.files_table(ui);
            });
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
