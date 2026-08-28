use slotmap::{SlotMap, new_key_type};

use crate::{core::ValidatedAlphabet, project::{Project, ProjectAction}, visual::{mappings::setup_visuals, render_menu, router::{Page, TinkerAction, render_tinker}}};

new_key_type! {
    pub struct ProjectKey;
}

pub struct App {
    pub current_page: Page,
    pub projects: SlotMap<ProjectKey, Project>,
    pub current_project: Option<ProjectKey>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(setup_visuals());

        let mut app = Self {
            current_page: Page::Menu,
            projects: SlotMap::with_key(),
            current_project: None,
        };

        match app.load_from_json(
            r#"
{"name":"","submachines":[{"name":"Main machine","nodes":[{"action":{"Left":1},"position":{"x":525.0,"y":375.0}},{"action":"Start","position":{"x":300.0,"y":375.0}},{"action":"Start","position":{"x":750.0,"y":375.0}}],"edges":[{"chars":["1"," "],"source":1,"target":0},{"chars":[" ","1"],"source":0,"target":2}],"notes":[]},{"name":"machine","nodes":[{"action":"Start","position":{"x":300.0,"y":450.0}},{"action":{"Left":1},"position":{"x":525.0,"y":300.0}},{"action":{"Right":1},"position":{"x":525.0,"y":600.0}},{"action":"Start","position":{"x":750.0,"y":450.0}}],"edges":[{"chars":["1"," "],"source":0,"target":1},{"chars":["1"," "],"source":1,"target":3},{"chars":["1"," "],"source":0,"target":2},{"chars":["1"," "],"source":2,"target":3}],"notes":[]}],"alphabet":[" ","1"]}
            "#.to_owned()
            ) {
            Ok(_) => (),
            Err(e) => eprintln!("{:?}", e),
        };

        app
    }

    pub fn current_project(&self) -> Option<&Project> {
        let key = self.current_project?;
        self.projects.get(key)
    }

    pub fn current_project_mut(&mut self) -> Option<&mut Project> {
        let key = self.current_project?;
        self.projects.get_mut(key)
    }

    fn create_project(&mut self, name: String, alphabet: ValidatedAlphabet) {
        self.current_project = Some(self.projects.insert(Project::new(name, alphabet)));
    }

    pub fn go_to_menu(&mut self) {
        self.current_page = Page::Menu;
        self.current_project = None;
    }

    pub fn load_from_json(&mut self, json: String) -> Result<(), serde_json::Error> {
        let project = Project::from_json(json)?;
        self.current_project = Some(
            self.projects.insert(project)
            );

        Ok(())
    }

    pub fn execute_project_action(&mut self, project_action: ProjectAction) {
        match project_action {
            ProjectAction::LoadFromMemory(key ) => {
                self.current_project = Some(key);
                self.current_page = Page::Tinkering;
            },
            ProjectAction::CreateProject((name, alphabet)) => {
                self.create_project(name, alphabet);
            }
            ProjectAction::CloseProject => {
                self.current_project = None;
                self.current_page = Page::Menu;
            }
            _ => unimplemented!()
        }
    }
}

impl eframe::App for App {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // // eframe::set_value(storage, "current_page", &self.current_page);
        // eframe::set_value(storage, "current_project_name", &self.current_project_name);
        // eframe::set_value(storage, "current_submachine", &self.current_submachine);
        //
        // if let Err(e) = crate::persistence::save_project(&project) {
        //     eprintln!("Ошибка автосохранения проекта при выходе: {e}");
        // }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::S)) {
        //     if !self.current_project_name.trim().is_empty() {
        //         let project = self.build_current_project();
        //         if let Err(e) = crate::persistence::save_project(&project) {
        //             self.file_status = Some(format!("Save error: {e}"));
        //         } else {
        //             self.file_status = Some("Saved".into());
        //             self.refresh_projects_cache();
        //         }
        //     }
        // }

        if let Some(key) = self.current_project {
            if let Some(project) = self.projects.get_mut(key) {
                let tinker_action = render_tinker(project, ctx);
                match tinker_action {
                    TinkerAction::GoToMenu => {
                        self.go_to_menu();
                    },
                    TinkerAction::SaveProject => {},
                    TinkerAction::None => {}
                }
            }
        } else {
            render_menu(self, ctx);
        }
    }
}

