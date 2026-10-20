use crate::core::alphabet::ValidatedAlphabet;
use crate::tu4::model::Tu4Document;
use crate::{
    core::{tape::Tape, test_case::TestCase},
    diagram::model::{
        document::{DiagramDocument, SubmachineDocument},
        submachine::{SubmachineKey, SubmachineName},
    },
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProjectDocument {
    Diagram(DiagramDocument),
    Tu4(Tu4Document),
}

impl ProjectDocument {
    pub fn diagram_mut(&mut self) -> Result<&mut DiagramDocument, String> {
        match self {
            Self::Diagram(document) => Ok(document),
            Self::Tu4(_) => Err("Diagram editing requires a diagram project".into()),
        }
    }
    pub fn alphabet(&self) -> &ValidatedAlphabet {
        match self {
            Self::Diagram(document) => document.alphabet(),
            Self::Tu4(document) => &document.alphabet,
        }
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum ProjectNameError {
    #[error("Name cannot be empty.")]
    EmptyName,
    #[error("Name cannot have more than 20 characters.")]
    NameTooLong,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectName(String);
impl ProjectName {
    pub fn new(name: String) -> Result<Self, ProjectNameError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(ProjectNameError::EmptyName);
        }
        if name.chars().count() > 20 {
            return Err(ProjectNameError::NameTooLong);
        }
        Ok(Self(name.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectData {
    pub name: ProjectName,
    pub document: ProjectDocument,
    pub created_at: u64,
    pub prepared_tape: Tape,
    pub tests: [Option<TestCase>; 6],
    pub step_limit: u64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TransactionKey {
    Canvas(SubmachineKey),
    PreparedTape,
    Source,
}

pub enum ProjectEdit {
    SetSource(Arc<str>),
    ReplaceMachine {
        key: SubmachineKey,
        machine: Box<SubmachineDocument>,
    },
    AddMachine(SubmachineName),
    RemoveMachine(SubmachineKey),
    RenameMachine {
        key: SubmachineKey,
        name: SubmachineName,
    },
    SetTape(Tape),
    SetTest {
        slot: usize,
        test: Option<TestCase>,
    },
    SetStepLimit(u64),
}
impl ProjectEdit {
    pub fn label(&self) -> &'static str {
        match self {
            Self::SetSource(_) => "Edit quadruples",
            Self::ReplaceMachine { .. } => "Edit diagram",
            Self::AddMachine(_) => "Create submachine",
            Self::RemoveMachine(_) => "Delete submachine",
            Self::RenameMachine { .. } => "Rename submachine",
            Self::SetTape(_) => "Edit tape",
            Self::SetTest { .. } => "Edit test",
            Self::SetStepLimit(_) => "Change test step limit",
        }
    }
}
struct HistoryEntry {
    data: ProjectData,
    label: String,
}
struct Pending {
    before: ProjectData,
    key: TransactionKey,
    label: String,
}

pub struct Project {
    data: ProjectData,
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
    pending: Option<Pending>,
}
impl Project {
    pub fn new(name: ProjectName, document: DiagramDocument) -> Self {
        Self::from_data(ProjectData {
            name,
            document: ProjectDocument::Diagram(document),
            created_at: timestamp(),
            prepared_tape: Tape::new(),
            tests: std::array::from_fn(|_| None),
            step_limit: 100_000,
        })
    }
    pub fn new_tu4(name: ProjectName, alphabet: ValidatedAlphabet) -> Self {
        Self::from_data(ProjectData {
            name,
            document: ProjectDocument::Tu4(Tu4Document::new(alphabet)),
            created_at: timestamp(),
            prepared_tape: Tape::new(),
            tests: std::array::from_fn(|_| None),
            step_limit: 100_000,
        })
    }
    pub fn is_tu4(&self) -> bool {
        matches!(self.data.document, ProjectDocument::Tu4(_))
    }
    pub fn alphabet(&self) -> &ValidatedAlphabet {
        self.data.document.alphabet()
    }
    pub fn tu4(&self) -> Option<&Tu4Document> {
        match &self.data.document {
            ProjectDocument::Tu4(document) => Some(document),
            ProjectDocument::Diagram(_) => None,
        }
    }
    pub fn from_data(data: ProjectData) -> Self {
        Self {
            data,
            undo: Vec::new(),
            redo: Vec::new(),
            pending: None,
        }
    }
    pub fn data(&self) -> &ProjectData {
        &self.data
    }
    pub fn name(&self) -> &str {
        self.data.name.as_str()
    }
    pub fn document(&self) -> &DiagramDocument {
        match &self.data.document {
            ProjectDocument::Diagram(document) => document,
            ProjectDocument::Tu4(_) => panic!("diagram access requires a diagram project"),
        }
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty() || self.pending.is_some()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty() && self.pending.is_none()
    }
    pub fn undo_label(&self) -> Option<&str> {
        self.pending
            .as_ref()
            .map(|p| p.label.as_str())
            .or_else(|| self.undo.last().map(|e| e.label.as_str()))
    }
    pub fn redo_label(&self) -> Option<&str> {
        self.redo.last().map(|e| e.label.as_str())
    }

    pub fn execute(
        &mut self,
        edit: ProjectEdit,
        group: Option<TransactionKey>,
    ) -> Result<bool, String> {
        if self
            .pending
            .as_ref()
            .map(|p| Some(p.key) != group)
            .unwrap_or(false)
        {
            self.finish_transaction();
        }
        let before = self.data.clone();
        let label = edit.label().to_owned();
        match edit {
            ProjectEdit::SetSource(source) => {
                let ProjectDocument::Tu4(document) = &mut self.data.document else {
                    return Err("Source editing requires a TU4 project".into());
                };
                document.source = source;
            }
            ProjectEdit::ReplaceMachine { key, machine } => {
                let document = self.data.document.diagram_mut()?;
                document.replace_submachine(key, *machine)?;
                if let Err(error) = document.validate_calls() {
                    self.data = before;
                    return Err(error);
                }
            }
            ProjectEdit::AddMachine(name) => {
                self.data.document.diagram_mut()?.add_submachine(name);
            }
            ProjectEdit::RemoveMachine(key) => {
                self.data.document.diagram_mut()?.remove_submachine(key)?;
            }
            ProjectEdit::RenameMachine { key, name } => {
                let machine = self
                    .data
                    .document
                    .diagram_mut()?
                    .submachine_mut(key)
                    .ok_or("Submachine no longer exists")?;
                if machine.is_system() {
                    return Err("System machines are read-only".into());
                }
                machine.machine_mut().rename(name);
            }
            ProjectEdit::SetTape(tape) => {
                tape.validate(self.alphabet())?;
                self.data.prepared_tape = tape;
            }
            ProjectEdit::SetTest { slot, test } => {
                if let Some(test) = &test {
                    test.initial_tape.validate(self.alphabet())?;
                    test.expected_tape.validate(self.alphabet())?;
                }
                *self.data.tests.get_mut(slot).ok_or("Invalid test slot")? = test;
            }
            ProjectEdit::SetStepLimit(limit) => self.data.step_limit = limit.clamp(1, 10_000_000),
        }
        if self.data == before {
            return Ok(false);
        }
        self.redo.clear();
        if let Some(key) = group {
            if self.pending.is_none() {
                self.pending = Some(Pending { before, key, label });
            }
        } else {
            self.push_undo(HistoryEntry {
                data: before,
                label,
            });
        }
        Ok(true)
    }
    fn push_undo(&mut self, entry: HistoryEntry) {
        const HISTORY_LIMIT: usize = 100;
        self.undo.push(entry);
        if self.undo.len() > HISTORY_LIMIT {
            self.undo.remove(0);
        }
    }
    pub fn finish_transaction(&mut self) {
        if let Some(pending) = self.pending.take() {
            if pending.before != self.data {
                self.push_undo(HistoryEntry {
                    data: pending.before,
                    label: pending.label,
                });
            }
        }
    }
    pub fn undo(&mut self) -> bool {
        self.finish_transaction();
        let Some(entry) = self.undo.pop() else {
            return false;
        };
        self.redo.push(HistoryEntry {
            data: std::mem::replace(&mut self.data, entry.data),
            label: entry.label,
        });
        true
    }
    pub fn redo(&mut self) -> bool {
        self.finish_transaction();
        let Some(entry) = self.redo.pop() else {
            return false;
        };
        let previous = std::mem::replace(&mut self.data, entry.data);
        self.push_undo(HistoryEntry {
            data: previous,
            label: entry.label,
        });
        true
    }
    pub fn validate(data: &ProjectData) -> Result<(), String> {
        ProjectName::new(data.name.as_str().to_owned()).map_err(|e| e.to_string())?;
        match &data.document {
            ProjectDocument::Diagram(document) => document.validate()?,
            ProjectDocument::Tu4(document) => {
                ValidatedAlphabet::new(document.alphabet.iter().copied())
                    .map_err(|e| e.to_string())?;
            }
        }
        let alphabet = data.document.alphabet();
        data.prepared_tape.validate(alphabet)?;
        for test in data.tests.iter().flatten() {
            test.initial_tape.validate(alphabet)?;
            test.expected_tape.validate(alphabet)?;
        }
        if !(1..=10_000_000).contains(&data.step_limit) {
            return Err("Invalid test step limit".into());
        }
        Ok(())
    }
}

pub fn timestamp() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as u64
    }
}
