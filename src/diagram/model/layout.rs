use slotmap::SecondaryMap;
use std::ops::{Add, AddAssign, Mul, Sub};

use serde::{Deserialize, Serialize};
use slotmap::{SlotMap, new_key_type};

use crate::diagram::model::submachine::NodeKey;

new_key_type! {
    pub struct NoteKey;
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LayoutPos {
    pub x: f32,
    pub y: f32,
}

impl LayoutPos {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Default for LayoutPos {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl Add for LayoutPos {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for LayoutPos {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl AddAssign<LayoutPos> for LayoutPos {
    fn add_assign(&mut self, rhs: LayoutPos) {
        self.x = self.x + rhs.x;
        self.y = self.y + rhs.y;
    }
}

impl Mul<f32> for LayoutPos {
    type Output = Self;

    fn mul(self, other: f32) -> Self {
        LayoutPos {
            x: self.x * other,
            y: self.y * other,
        }
    }
}

impl From<LayoutPos> for egui::Pos2 {
    fn from(pos: LayoutPos) -> Self {
        egui::Pos2::new(pos.x, pos.y)
    }
}

impl From<egui::Pos2> for LayoutPos {
    fn from(pos: egui::Pos2) -> Self {
        LayoutPos { x: pos.x, y: pos.y }
    }
}

impl From<LayoutPos> for egui::Vec2 {
    fn from(pos: LayoutPos) -> Self {
        egui::Vec2::new(pos.x, pos.y)
    }
}

impl From<egui::Vec2> for LayoutPos {
    fn from(pos: egui::Vec2) -> Self {
        LayoutPos { x: pos.x, y: pos.y }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub position: LayoutPos,
    pub content: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GraphLayout {
    node_positions: SecondaryMap<NodeKey, LayoutPos>,
    notes: SlotMap<NoteKey, Note>,
}

impl PartialEq for GraphLayout {
    fn eq(&self, other: &Self) -> bool {
        self.node_positions == other.node_positions && self.notes.iter().eq(other.notes.iter())
    }
}

impl GraphLayout {
    pub const GRID_SIZE: f32 = 75.0;

    pub fn new() -> Self {
        Self {
            node_positions: SecondaryMap::new(),
            notes: SlotMap::with_key(),
        }
    }

    pub fn snap_to_grid(position: LayoutPos) -> LayoutPos {
        LayoutPos {
            x: (position.x / Self::GRID_SIZE).round() * Self::GRID_SIZE,
            y: (position.y / Self::GRID_SIZE).round() * Self::GRID_SIZE,
        }
    }

    pub fn get_node_position(&self, node_key: NodeKey) -> Option<LayoutPos> {
        self.node_positions.get(node_key).copied()
    }

    pub fn update_node_position(&mut self, node_key: NodeKey) {
        if let Some(position) = self.node_positions.get_mut(node_key) {
            *position = GraphLayout::snap_to_grid(*position);
        }
    }

    pub fn update_note_position(&mut self, note_key: NoteKey) {
        if let Some(note) = self.notes.get_mut(note_key) {
            note.position = GraphLayout::snap_to_grid(note.position);
        }
    }

    pub fn set_node_position(&mut self, node_key: NodeKey, position: LayoutPos) {
        self.node_positions
            .insert(node_key, Self::snap_to_grid(position));
    }

    pub fn remove_node(&mut self, id: NodeKey) {
        self.node_positions.remove(id);
    }

    pub fn add_note(&mut self, position: LayoutPos, content: String) -> NoteKey {
        self.notes.insert(Note { position, content })
    }

    pub fn note_mut(&mut self, id: NoteKey) -> Option<&mut Note> {
        self.notes.get_mut(id)
    }

    pub fn remove_note(&mut self, id: NoteKey) {
        self.notes.remove(id);
    }

    pub fn update_node_position_unsnapped(&mut self, node_key: NodeKey, delta: LayoutPos) {
        if let Some(note) = self.node_positions.get_mut(node_key) {
            note.x += delta.x;
            note.y += delta.y;
        }
    }

    pub fn move_note_unsnapped(&mut self, key: NoteKey, delta: LayoutPos) {
        if let Some(note) = self.notes.get_mut(key) {
            note.position += delta;
        }
    }

    pub fn notes_iter(&self) -> impl Iterator<Item = (NoteKey, &Note)> {
        self.notes.iter()
    }

    pub fn nodes_iter(&self) -> impl Iterator<Item = (NodeKey, &LayoutPos)> {
        self.node_positions.iter()
    }
}
