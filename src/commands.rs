use crate::{
    app::ProjectKey,
    core::{alphabet::ValidatedAlphabet, tape::Tape},
    diagram::{editor::tool::EditorTool, model::submachine::SubmachineKey},
    project::model::{ProjectEdit, ProjectName, TransactionKey},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CommandId {
    Cheatsheet,
    NewProject,
    OpenProject,
    DeleteProject,
    Import,
    Export,
    Save,
    Menu,
    Undo,
    Redo,
    Validate,
    OpenRunner,
    BackToEditor,
    Run,
    Pause,
    Step,
    Reset,
    Stop,
    EditDiagram,
    EditSource,
    OpenTu4,
    ToggleBreakpoint,
    AddMachine,
    RemoveMachine,
    RenameMachine,
    OpenMachine,
    MachineSettings,
    PlaceMachine,
    EditTape,
    EditTest,
    LoadTest,
    CheckAll,
    CancelChecks,
    StepLimit,
    TapeLeftFive,
    TapeLeft,
    TapeRight,
    TapeRightFive,
    TapeHead,
    Move,
    Link,
    Note,
    Delete,
    Start,
    End,
    Left,
    Right,
    Write,
}
pub struct CommandInfo {
    pub id: CommandId,
    pub label: &'static str,
    pub icon: &'static str,
    pub key: Option<egui::Key>,
    pub modified: bool,
}
use CommandId::*;
macro_rules! entry {
    ($id:ident, $label:literal, $icon:literal) => {
        CommandInfo {
            id: $id,
            label: $label,
            icon: $icon,
            key: None,
            modified: false,
        }
    };
    ($id:ident, $label:literal, $icon:literal, $key:ident, $modified:literal) => {
        CommandInfo {
            id: $id,
            label: $label,
            icon: $icon,
            key: Some(egui::Key::$key),
            modified: $modified,
        }
    };
}
pub static REGISTRY: &[CommandInfo] = &[
    entry!(Cheatsheet, "Cheatsheet", "?", F1, false),
    entry!(NewProject, "New project", "+", N, true),
    entry!(OpenProject, "Open", "↗"),
    entry!(DeleteProject, "Delete project", "×"),
    entry!(Import, "Import project", "↓", O, true),
    entry!(Export, "Export project", "↑"),
    entry!(Save, "Save", "▣", S, true),
    entry!(Menu, "Projects", "⌂"),
    entry!(Undo, "Undo", "↶", Z, true),
    entry!(Redo, "Redo", "↷", Y, true),
    entry!(Validate, "Validate", "✓", V, false),
    entry!(OpenRunner, "Run project", "▶", P, false),
    entry!(BackToEditor, "Editor", "←"),
    entry!(Run, "Run", "▶", P, false),
    entry!(Pause, "Pause", "Ⅱ", P, false),
    entry!(Step, "Step", "▷", Space, false),
    entry!(Reset, "Reset", "↺", C, true),
    entry!(Stop, "Stop", "■", D, true),
    entry!(EditDiagram, "Edit diagram", "◇"),
    entry!(EditSource, "Edit quadruples", "✎"),
    entry!(ToggleBreakpoint, "Toggle breakpoint", "●"),
    entry!(OpenTu4, "Open project in TU4", "↗"),
    entry!(AddMachine, "New submachine", "+", N, true),
    entry!(RemoveMachine, "Delete submachine", "×"),
    entry!(RenameMachine, "Rename submachine", "✎"),
    entry!(OpenMachine, "Open submachine", "↗"),
    entry!(MachineSettings, "Submachine settings", "⚙"),
    entry!(PlaceMachine, "Place submachine", "◇"),
    entry!(EditTape, "Edit tape", "✎"),
    entry!(EditTest, "Edit test", "✎"),
    entry!(LoadTest, "Load test", "↓"),
    entry!(CheckAll, "Check all six", "✓"),
    entry!(CancelChecks, "Cancel checks", "■"),
    entry!(StepLimit, "Step limit", "#"),
    entry!(TapeLeftFive, "View 5 cells left", "≪"),
    entry!(TapeLeft, "View 1 cell left", "‹"),
    entry!(TapeRight, "View 1 cell right", "›"),
    entry!(TapeRightFive, "View 5 cells right", "≫"),
    entry!(TapeHead, "Return to head", "◎"),
    entry!(Move, "Move", "✋", M, false),
    entry!(Link, "Link", "↔", S, false),
    entry!(Note, "Note", "💬", B, false),
    entry!(Delete, "Delete", "❌", X, false),
    entry!(Start, "Start", "○", T, false),
    entry!(End, "End", "◎", G, false),
    entry!(Left, "Left", "l", A, false),
    entry!(Right, "Right", "r", D, false),
    entry!(Write, "Write", "λ", W, false),
];
impl CommandId {
    pub fn info(self) -> &'static CommandInfo {
        REGISTRY
            .iter()
            .find(|info| info.id == self)
            .expect("every command has metadata")
    }
}
impl CommandInfo {
    pub fn shortcut(&self) -> Option<String> {
        self.key.map(|key| {
            if self.modified {
                format!("Ctrl/⌘+{}", key.name())
            } else {
                key.name().into()
            }
        })
    }
}

pub enum Command {
    Invoke(CommandId),
    Create {
        name: ProjectName,
        alphabet: ValidatedAlphabet,
    },
    CreateTu4 {
        name: ProjectName,
        alphabet: ValidatedAlphabet,
    },
    Open(ProjectKey),
    DeleteProject(ProjectKey),
    Edit {
        edit: ProjectEdit,
        group: Option<TransactionKey>,
    },
    SelectTool(EditorTool),
    OpenMachine(SubmachineKey),
    LoadTest(usize),
    SetTape(Tape),
    ScrollTape(i32),
}

impl Command {
    pub fn id(&self) -> CommandId {
        match self {
            Self::Invoke(id) => *id,
            Self::Create { .. } | Self::CreateTu4 { .. } => NewProject,
            Self::Open(_) => OpenProject,
            Self::DeleteProject(_) => DeleteProject,
            Self::Edit { edit, .. } => match edit {
                ProjectEdit::SetSource(_) => EditSource,
                ProjectEdit::ToggleBreakpoint(_) => ToggleBreakpoint,
                ProjectEdit::ReplaceMachine { .. } => EditDiagram,
                ProjectEdit::AddMachine(_) => AddMachine,
                ProjectEdit::RemoveMachine(_) => RemoveMachine,
                ProjectEdit::RenameMachine { .. } => RenameMachine,
                ProjectEdit::SetTape(_) => EditTape,
                ProjectEdit::SetTest { .. } => EditTest,
                ProjectEdit::SetStepLimit(_) => StepLimit,
            },
            Self::SelectTool(_) => Move,
            Self::OpenMachine(_) => OpenMachine,
            Self::LoadTest(_) => LoadTest,
            Self::SetTape(_) => EditTape,
            Self::ScrollTape(_) => TapeLeft,
        }
    }
}

pub fn button(ui: &mut egui::Ui, id: CommandId, enabled: bool) -> egui::Response {
    let info = id.info();
    let response = ui.add_enabled(enabled, egui::Button::new(format!("     {}", info.label)));
    let color = if enabled {
        ui.visuals().text_color()
    } else {
        ui.visuals().weak_text_color()
    };
    crate::ui::widgets::custom_buttons::paint_icon(
        ui.painter(),
        egui::pos2(response.rect.left() + 14., response.rect.center().y),
        14.,
        info.icon,
        color,
    );
    if let Some(shortcut) = info.shortcut() {
        response.on_hover_text(shortcut)
    } else {
        response
    }
}

pub fn icon_button(ui: &mut egui::Ui, id: CommandId) -> egui::Response {
    let response = ui.add_sized([30., 28.], egui::Button::new(""));
    crate::ui::widgets::custom_buttons::paint_icon(
        ui.painter(),
        response.rect.center(),
        16.,
        id.info().icon,
        ui.visuals().text_color(),
    );
    response.on_hover_text(id.info().label)
}
