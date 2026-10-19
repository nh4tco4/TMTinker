pub mod project_to_json;
pub mod transfer;

use crate::project::{Project, model::ProjectData};
use serde::{Deserialize, Serialize};

pub const STORAGE_KEY: &str = "tmtinker.projects.v1";
#[derive(Serialize, Deserialize)]
struct Library {
    version: u32,
    projects: Vec<ProjectData>,
}
#[derive(Serialize, Deserialize)]
struct ProjectFile {
    version: u32,
    project: ProjectData,
}

pub fn encode_library<'a>(projects: impl Iterator<Item = &'a Project>) -> Result<String, String> {
    serde_json::to_string(&Library {
        version: 1,
        projects: projects.map(|p| p.data().clone()).collect(),
    })
    .map_err(|e| e.to_string())
}
pub fn decode_library(json: &str) -> Result<Vec<Project>, String> {
    let library: Library =
        serde_json::from_str(json).map_err(|e| format!("Cannot read saved projects: {e}"))?;
    if library.version != 1 {
        return Err("Unsupported project library version".into());
    }
    library
        .projects
        .into_iter()
        .map(|data| {
            Project::validate(&data)?;
            Ok(Project::from_data(data))
        })
        .collect()
}
pub fn encode_project(project: &Project) -> Result<String, String> {
    serde_json::to_string_pretty(&ProjectFile {
        version: 1,
        project: project.data().clone(),
    })
    .map_err(|e| e.to_string())
}
pub fn decode_project(json: &str) -> Result<Project, String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Cannot import project: {e}"))?;
    if value.get("version").is_none() {
        let legacy: project_to_json::ProjectBundle = serde_json::from_value(value)
            .map_err(|e| format!("Cannot import legacy project: {e}"))?;
        return legacy.into_project();
    }
    let file: ProjectFile =
        serde_json::from_value(value).map_err(|e| format!("Cannot import project: {e}"))?;
    if file.version != 1 {
        return Err("Unsupported project file version".into());
    }
    Project::validate(&file.project)?;
    Ok(Project::from_data(file.project))
}

pub fn creation_date(seconds: u64) -> String {
    let z = (seconds / 86400) as i64 + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02} UTC")
}
