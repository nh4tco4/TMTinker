// use crate::{
//     core::{graph::NodeKey, tm::Submachine},
//     editor::{layout::LayoutPos, notes::NotesLayer},
// };
// use serde::{Deserialize, Serialize};
// use slotmap::Key;
// use std::collections::HashMap;
//
// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct Project {
//     name: String,
//     submachines: Vec<SubmachineBundle>,
//     alphabet: Vec<char>,
// }
//
// impl Project {
//     pub fn new(name: impl Into<String>) -> Self {
//         let main_sm = Submachine::new("Main".into());
//         let bundle = SubmachineBundle::from_parts(
//             &main_sm,
//             std::collections::HashMap::new(),
//             NotesLayer::default(),
//         );
//
//         Self {
//             name: name.into(),
//             submachines: vec![bundle],
//             alphabet: vec!['1', '0', ' '],
//         }
//     }
// }
//
// pub fn project_to_json(project: &Project) -> Result<String, String> {
//     serde_json::to_string_pretty(project).map_err(|e| format!("Serialization error: {e}"))
// }
//
// pub fn project_from_json(json: &str) -> Result<Project, String> {
//     serde_json::from_str(json).map_err(|e| format!("Parse error: {e}"))
// }
//
// pub fn submachine_to_json(sm: &Submachine) -> Result<String, String> {
//     serde_json::to_string_pretty(sm).map_err(|e| format!("Serialization error: {e}"))
// }
//
// pub fn submachine_from_json(json: &str) -> Result<Submachine, String> {
//     serde_json::from_str(json).map_err(|e| format!("Parse error: {e}"))
// }
//
// // --- Native backend
//
// #[cfg(not(target_arch = "wasm32"))]
// mod native {
//     use super::{Project, project_from_json, project_to_json};
//     use crate::{
//         core::{graph::NodeKey, tm::Submachine},
//         editor::{layout::LayoutPos, notes::NotesLayer},
//         persistence::{SubmachineBundle, submachine_bundle_from_json, submachine_bundle_to_json},
//     };
//     use std::{collections::HashMap, fs, path::PathBuf};
//
//     fn data_dir() -> PathBuf {
//         #[cfg(target_os = "windows")]
//         let base = std::env::var("APPDATA")
//             .map(PathBuf::from)
//             .unwrap_or_else(|_| PathBuf::from("."))
//             .join("turing_machine");
//
//         #[cfg(not(target_os = "windows"))]
//         let base = PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
//             .join(".local")
//             .join("share")
//             .join("turing_machine");
//
//         base
//     }
//
//     pub fn projects_dir() -> PathBuf {
//         data_dir().join("projects")
//     }
//
//     fn sanitize_filename(name: &str) -> String {
//         name.chars()
//             .map(|c| match c {
//                 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.' => c,
//                 _ => '_',
//             })
//             .collect()
//     }
//
//     fn project_path(name: &str) -> PathBuf {
//         projects_dir().join(format!("{}.json", sanitize_filename(name)))
//     }
//
//     pub fn list_projects() -> Vec<(String, String)> {
//         let Ok(entries) = fs::read_dir(projects_dir()) else {
//             return vec![];
//         };
//
//         let mut list = Vec::new();
//         for entry in entries.flatten() {
//             let path = entry.path();
//             if path.extension().and_then(|s| s.to_str()) != Some("json") {
//                 continue;
//             }
//             if let Ok(json) = fs::read_to_string(&path) {
//                 if let Ok(project) = project_from_json(&json) {
//                     let alphabet_str = project.alphabet.iter().collect::<String>();
//                     list.push((project.name, alphabet_str));
//                 }
//             }
//         }
//         list.sort_by(|a, b| a.0.cmp(&b.0));
//         list
//     }
//
//     pub fn save_project(project: &Project) -> Result<(), String> {
//         fs::create_dir_all(projects_dir()).map_err(|e| format!("Cannot create dir: {e}"))?;
//         let json = project_to_json(project)?;
//         fs::write(project_path(&project.name), json).map_err(|e| format!("Write error: {e}"))
//     }
//
//     pub fn load_project(name: &str) -> Result<Project, String> {
//         let json =
//             fs::read_to_string(project_path(name)).map_err(|e| format!("Read error: {e}"))?;
//         project_from_json(&json)
//     }
//
//     pub fn delete_project(name: &str) -> Result<(), String> {
//         let path = project_path(name);
//         if !path.starts_with(projects_dir()) {
//             return Err("Invalid project path".to_string());
//         }
//         fs::remove_file(path).map_err(|e| format!("Delete error: {e}"))
//     }
//
//     pub fn export_project_to(project: &Project, path: &std::path::Path) -> Result<(), String> {
//         let json = project_to_json(project)?;
//        fs::write(path, json).map_err(|e| format!("Write error: {e}"))
//     }
//
//     pub fn export_submachine_with_layout(
//         sm: &Submachine,
//         node_positions: &HashMap<NodeKey, LayoutPos>,
//         notes: &NotesLayer,
//     ) -> Result<String, String> {
//         let bundle = SubmachineBundle::from_parts(sm, node_positions.clone(), notes.clone());
//         submachine_bundle_to_json(&bundle)
//     }
//
//     pub fn import_submachine_with_layout(json: &str) -> Result<SubmachineBundle, String> {
//         submachine_bundle_from_json(json)
//     }
// }
//
// // --- WASM backend (Trunk)
//
// #[cfg(target_arch = "wasm32")]
// mod wasm {
//     use super::{Project, project_from_json, project_to_json, submachine_to_json};
//     use crate::core::{graph::NodeKey, tm::Submachine};
//     use crate::editor::layout::LayoutPos;
//     use crate::editor::notes::NotesLayer;
//     use crate::persistence::{
//         SubmachineBundle, submachine_bundle_from_json, submachine_bundle_to_json,
//     };
//     use std::collections::HashMap;
//     use wasm_bindgen::JsCast;
//
//     const PREFIX: &str = "tm_project:";
//
//     fn storage() -> Option<web_sys::Storage> {
//         web_sys::window()?.local_storage().ok()?
//     }
//
//     pub fn list_projects() -> Vec<(String, String, bool)> {
//         let Some(store) = storage() else {
//             return vec![];
//         };
//         let len = store.length().unwrap_or(0);
//         let mut list = Vec::new();
//
//         for i in 0..len {
//             if let Ok(Some(key)) = store.key(i) {
//                 if let Some(name) = key.strip_prefix(PREFIX) {
//                     if let Ok(Some(json)) = store.get_item(&key) {
//                         if let Ok(project) = project_from_json(&json) {
//                             let alphabet_str = project.alphabet.iter().collect::<String>();
//                             list.push((
//                                 name.to_string(),
//                                 alphabet_str,
//                             ));
//                         }
//                     }
//                 }
//             }
//         }
//         list.sort_by(|a, b| a.0.cmp(&b.0));
//         list
//     }
//
//     pub fn save_project(project: &Project) -> Result<(), String> {
//         let json = project_to_json(project)?;
//         storage()
//             .ok_or_else(|| "localStorage unavailable".to_owned())?
//             .set_item(&format!("{PREFIX}{}", project.name), &json)
//             .map_err(|e| format!("localStorage error: {e:?}"))
//     }
//
//     pub fn load_project(name: &str) -> Result<Project, String> {
//         let json = storage()
//             .ok_or_else(|| "localStorage unavailable".to_owned())?
//             .get_item(&format!("{PREFIX}{name}"))
//             .map_err(|e| format!("localStorage error: {e:?}"))?
//             .ok_or_else(|| format!("Project «{name}» not found"))?;
//         project_from_json(&json)
//     }
//
//     pub fn delete_project(name: &str) -> Result<(), String> {
//         storage()
//             .ok_or_else(|| "localStorage unavailable".to_owned())?
//             .remove_item(&format!("{PREFIX}{name}"))
//             .map_err(|e| format!("localStorage error: {e:?}"))
//     }
//
//     pub fn download_project(project: &Project) -> Result<(), String> {
//         let json = project_to_json(project)?;
//         download_text(&json, &format!("{}.json", sanitize_filename(&project.name)))
//     }
//
//     pub fn download_submachine(
//         sm: &Submachine,
//         node_positions: &HashMap<NodeKey, LayoutPos>,
//         notes: &NotesLayer,
//     ) -> Result<(), String> {
//         let json = export_submachine_with_layout(sm, node_positions, notes)?;
//         let safe_name = sanitize_filename(&sm.name);
//         download_text(&json, &format!("submachine_{}.json", safe_name))
//     }
//
//     pub fn download_all_submachines_with_layout(
//         submachines: &[(Submachine, HashMap<NodeKey, LayoutPos>, NotesLayer)],
//     ) -> Result<(), String> {
//         for (idx, (sm, positions, notes)) in submachines.iter().enumerate() {
//             let json = export_submachine_with_layout(sm, positions, notes)?;
//             let safe_name = sanitize_filename(&sm.name);
//             download_text(&json, &format!("submachine_{}_{}.json", idx, safe_name))?;
//         }
//         Ok(())
//     }
//
//     fn sanitize_filename(name: &str) -> String {
//         name.chars()
//             .map(|c| match c {
//                 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-' | '.' => c,
//                 _ => '_',
//             })
//             .collect()
//     }
//
//     fn download_text(text: &str, filename: &str) -> Result<(), String> {
//         let window = web_sys::window().ok_or("No window")?;
//         let document = window.document().ok_or("No document")?;
//
//         let parts = js_sys::Array::new();
//         parts.push(&wasm_bindgen::JsValue::from_str(text));
//         let opts = web_sys::BlobPropertyBag::new();
//         opts.set_type("application/json");
//         let blob = web_sys::Blob::new_with_str_sequence_and_options(&parts, &opts)
//             .map_err(|e| format!("Blob error: {e:?}"))?;
//
//         let url = web_sys::Url::create_object_url_with_blob(&blob)
//             .map_err(|e| format!("URL error: {e:?}"))?;
//
//         let anchor = document
//             .create_element("a")
//             .map_err(|e| format!("createElement error: {e:?}"))?;
//
//         let a: web_sys::HtmlAnchorElement = anchor
//             .dyn_into()
//             .map_err(|_| "Failed to cast Element to HtmlAnchorElement".to_string())?;
//
//         a.set_href(&url);
//         a.set_download(filename);
//         a.click();
//
//         let _ = web_sys::Url::revoke_object_url(&url);
//         Ok(())
//     }
//
//     pub fn export_submachine_with_layout(
//         sm: &Submachine,
//         node_positions: &HashMap<NodeKey, LayoutPos>,
//         notes: &NotesLayer, // ДОБАВЬТЕ ПАРАМЕТР
//     ) -> Result<String, String> {
//         let bundle = SubmachineBundle::from_parts(sm, node_positions.clone(), notes.clone());
//         submachine_bundle_to_json(&bundle)
//     }
//
//     pub fn import_submachine_with_layout(json: &str) -> Result<SubmachineBundle, String> {
//         submachine_bundle_from_json(json)
//     }
// }
//
// // --- Public re-exports
//
// #[cfg(not(target_arch = "wasm32"))]
// pub use native::{
//     delete_project, export_submachine_with_layout, import_submachine_with_layout, list_projects,
//     load_project, save_project,
// };
//
// #[cfg(target_arch = "wasm32")]
// pub use wasm::{
//     delete_project, download_all_submachines_with_layout, download_project, download_submachine,
//     export_submachine_with_layout, import_submachine_with_layout, list_projects, load_project,
//     save_project,
// };
//
// // --- SubmachineBundle
//
// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct SubmachineBundle {
//     pub submachine: Submachine,
//     pub node_positions: Vec<(u64, LayoutPos)>,
//     pub notes: NotesLayer,
// }
//
// impl SubmachineBundle {
//     pub fn from_parts(
//         sm: &Submachine,
//         node_positions: HashMap<NodeKey, LayoutPos>,
//         notes: NotesLayer,
//     ) -> Self {
//         Self {
//             submachine: sm.clone(),
//             node_positions: node_positions
//                 .iter()
//                 .map(|(k, pos)| (k.data().as_ffi(), *pos))
//                 .collect(),
//             notes,
//         }
//     }
//
//     pub fn apply_to(self, target_sm: &mut Submachine) -> (HashMap<NodeKey, LayoutPos>, NotesLayer) {
//         *target_sm = self.submachine;
//         let mut restored = HashMap::default();
//
//         let position_map: HashMap<u64, LayoutPos> = self.node_positions.into_iter().collect();
//
//         for (key, _node) in target_sm.graph.nodes.iter() {
//             if let Some(pos) = position_map.get(&key.data().as_ffi()) {
//                 restored.insert(key, *pos);
//             }
//         }
//         (restored, self.notes)
//     }
// }
//
// pub fn submachine_bundle_to_json(bundle: &SubmachineBundle) -> Result<String, String> {
//     serde_json::to_string_pretty(bundle).map_err(|e| format!("Serialization error: {e}"))
// }
//
// pub fn submachine_bundle_from_json(json: &str) -> Result<SubmachineBundle, String> {
//     serde_json::from_str(json).map_err(|e| format!("Parse error: {e}"))
// }
