use egui::Color32;
use std::io;
use std::path::PathBuf;

use crate::File;
use crate::Project;
use crate::TaskTreeNode;

pub const SPACING: f32 = 5.;
const TEST_PROJECT_PATH_WIN: &str = "D:\\Dropbox (Personal)\\Annat\\Kod\\rclamp\\test_folder";
const TEST_PROJECT_PATH_MAC: &str =
    "/Users/johnbuddee/Dropbox (Personal)/Annat/Kod/rclamp/test_folder";

#[derive(serde::Deserialize, serde::Serialize)]
struct RclampConfig {
    dark_mode: bool,
    projects_dir: PathBuf,
    template_project: Project,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Rclamp {
    current_project: Project,
    current_project_task_tree: TaskTreeNode,
    current_task: TaskTreeNode,
    projects: Vec<Project>,
    files: Vec<File>,
    config: RclampConfig,

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
}

impl Default for Rclamp {
    fn default() -> Self {
        let test_project_path = if cfg!(windows) {
            TEST_PROJECT_PATH_WIN
        } else {
            TEST_PROJECT_PATH_MAC
        };
        let projects_dir = PathBuf::from(test_project_path);

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
        let projects = match Project::find_projects(projects_dir, template_project.clone()) {
            Ok(p) => p,
            Err(error) => panic!("Error when looking for projects: {}", error),
        };

        let empty_task = TaskTreeNode::new(String::new(), PathBuf::new());

        Self {
            current_project: template_project.clone(),
            projects: projects,
            current_project_task_tree: empty_task.clone(),
            current_task: empty_task.clone(),
            files: Vec::new(),
            config: RclampConfig {
                dark_mode: true,
                projects_dir: PathBuf::from(test_project_path),
                template_project: template_project,
            },

            warning_message: String::new(),
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
        }
    }
}

impl Rclamp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    /// Simply sets the current project.
    fn set_current_project(&mut self, project: Project) {
        self.current_project = project;
    }

    pub fn set_current_task(&mut self, task: TaskTreeNode) {
        self.current_task = task;
        self.files = self
            .current_task
            .find_workfiles(String::from("01_work"))
            .unwrap_or(Vec::new());
        self.files.sort();
        self.files.reverse();
    }

    fn refresh_all(&mut self, ui: &mut egui::Ui) {
        self.refresh_projects();
        self.refresh_tasks(ui);
        self.refresh_files();
    }

    /// Refreshes the list of projects by calling find_projects.
    fn refresh_projects(&mut self) {
        self.projects = match Project::find_projects(
            self.config.projects_dir.clone(),
            self.config.template_project.clone(),
        ) {
            Ok(p) => p,
            Err(e) => panic!("Error when looking for projects: {}", e),
        };
    }

    /// Refreshes task tree.
    fn refresh_tasks(&mut self, ui: &mut egui::Ui) {
        let tree = match TaskTreeNode::from_path(self.current_project.clone().work_dir) {
            Ok(t) => t,
            Err(e) => {
                self.render_task_tree_error(ui, e);
                return;
            }
        };
        self.current_project_task_tree = tree;
    }

    /// Refreshes file list.
    fn refresh_files(&mut self) {
        self.set_current_task(self.current_task.clone());
    }

    /// Renders the list of projects.
    fn render_projects(&mut self, ui: &mut egui::Ui) {
        let projects = &self.projects.clone();

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
        self.set_current_project(project);
        let tree = match TaskTreeNode::from_path(self.current_project.clone().work_dir) {
            Ok(t) => t,
            Err(e) => {
                self.render_task_tree_error(ui, e);
                return;
            }
        };
        self.current_project_task_tree = tree;
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
            ui.label(&self.new_task_message);
            ui.label(egui::RichText::new(&self.new_task_error).color(Color32::RED));

            if cancel_btn.clicked() {
                self.show_create_task = false;
            }

            if create_task_btn.clicked() {
                match self
                    .new_task_parent
                    .create_task(self.new_task_name.clone(), self.current_project.clone())
                {
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
            ui.label(&self.new_folder_message);
            ui.label(egui::RichText::new(&self.new_folder_error).color(Color32::RED));

            if cancel_btn.clicked() {
                self.show_create_folder = false;
            }

            if create_folder_btn.clicked() {
                match self
                    .new_folder_parent
                    .create_task(self.new_folder_name.clone(), self.current_project.clone())
                {
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
            ui.label(&self.new_project_message);
            ui.label(egui::RichText::new(&self.new_project_error).color(Color32::RED));

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
                        ui.label(format!("{}", self.warning_message));
                    },
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    let close_btn = ui.add(egui::Button::new("❌"));
                    let refresh_btn = ui.add(egui::Button::new("🔄"));
                    let theme_btn = ui.add(egui::Button::new("🌙"));

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
        let task = self.current_project_task_tree.clone();
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
                for f in &self.files {
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

        if self.show_create_task {
            egui::TopBottomPanel::top("create_task_panel").show(ctx, |ui| {
                self.create_task_dialog(ui);
            });
        }

        if self.show_create_folder {
            egui::TopBottomPanel::top("create_folder_panel").show(ctx, |ui| {
                self.create_folder_dialog(ui);
            });
        }

        egui::SidePanel::left("first_left_panel").show(ctx, |ui| {
            // Left panel

            egui::ScrollArea::vertical().show(ui, |ui| {
                self.render_projects(ui);
            });
        });

        egui::SidePanel::left("second_left_panel").show(ctx, |ui| {
            // Middle panel

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(SPACING);
                ui.label(format!("Current project: {}", self.current_project.name));
                ui.add(egui::Separator::default());
                ui.add_space(SPACING);
                self.render_task_tree(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Right panel

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.label(format!("Current task: {}", self.current_task.name));
                ui.add(egui::Separator::default());
                ui.add_space(SPACING);
                self.files_table(ui);
            });
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
