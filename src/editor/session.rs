use egui::Vec2;

use crate::{core::tm::{EdgeKey, NodeKey}, editor::layout::{LayoutPos, NoteKey}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelectableKey {
    Node(NodeKey),
    Edge(EdgeKey),
    Note(NoteKey),
}

#[derive(Default, Clone)]
pub struct CanvasInteractionState {
    pub rect_select: Option<RectSelection>,
    pub selection_state: SelectionState,
    pub drag: Vec<SelectableKey>,
    pub editing_note: Option<NoteKey>,
    pub link_source: Option<NodeKey>,
}

#[derive(Default, Clone)]
pub struct RectSelection {
    pub start: egui::Pos2,
    pub current: egui::Pos2,
}

#[derive(Clone, Default)]
pub struct SelectionState {
    pub selected: Vec<SelectableKey>,
    pub hovered: Option<SelectableKey>,
}

impl SelectionState {
    pub fn is_selected(&self, id: &SelectableKey) -> bool {
        self.selected.contains(id)
    }

    pub fn toggle_selected(&mut self, id: SelectableKey) {
        if self.selected.contains(&id) {
            self.selected.retain(|s| s != &id);
        } else {
            self.selected.push(id);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    pub fn clear(&mut self) {
        self.hovered = None;
        self.selected.clear();
    }

    pub fn single_select(&mut self, id: SelectableKey) {
        self.selected.clear();
        self.selected.push(id);
    }
    
    pub fn single_note(&self) -> Option<NoteKey> {
        if self.selected.len() == 1 {
            match self.selected[0] {
                SelectableKey::Note(note_key) => Some(note_key),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn single_node(&self) -> Option<NodeKey> {
        if self.selected.len() == 1 {
            match self.selected[0] {
                SelectableKey::Node(node_key) => Some(node_key),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn single_edge(&self) -> Option<EdgeKey> {
        if self.selected.len() == 1 {
            match self.selected[0] {
                SelectableKey::Edge(edge_key) => Some(edge_key),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn selected_node_keys(&self) -> Vec<NodeKey> {
        self.selected
            .iter()
            .filter_map(|id| match id {
                SelectableKey::Node(node_key) => Some(*node_key),
                _ => None,
            })
            .collect()
    }
}

pub struct Zoom(f32);

impl Zoom {
    const MAX: f32 = 15.0;
    const MIN: f32 = 0.5;
    pub const UNIT: Zoom = Zoom(1.0);

    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    pub fn get(&self) -> f32 {
        self.0
    }
}

pub struct ViewportState {
    pub camera_pan: LayoutPos,
    pub camera_zoom: Zoom,
}

impl ViewportState {
    pub fn new() -> Self {
        Self {
            camera_pan: LayoutPos::new(0.0, 0.0),
            camera_zoom: Zoom::UNIT,
        }
    }

    /// Converts screen coordinates to world coordinates.
    pub fn screen_to_world(&self, screen_pos: LayoutPos) -> LayoutPos {
        let world_pos = LayoutPos {
            x: (screen_pos.x - self.camera_pan.x) / self.camera_zoom.get(),
            y: (screen_pos.y - self.camera_pan.y) / self.camera_zoom.get(),
        };
        world_pos
    }

    /// Converts world coordinates to screen coordinates.
    pub fn world_to_screen(&self, world_pos: LayoutPos) -> LayoutPos {
        let screen_pos = LayoutPos {
            x: world_pos.x * self.camera_zoom.get() + self.camera_pan.x,
            y: world_pos.y * self.camera_zoom.get() + self.camera_pan.y,
        };
        screen_pos
    }
}

