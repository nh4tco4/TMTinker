use std::ops::{Add, AddAssign, Mul, Sub};
use slotmap::SecondaryMap;

use serde::{Deserialize, Serialize};
use slotmap::{SlotMap, new_key_type};

use crate::core::tm::NodeKey;

new_key_type! {
    pub struct NoteKey;
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    pub position: LayoutPos,
    pub content: String,
}

#[derive(Clone)]
pub struct GraphLayout {
    node_positions: SecondaryMap<NodeKey, LayoutPos>,
    notes: SlotMap<NoteKey, Note>,
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

    pub fn shift_node(&mut self, node_key: NodeKey, shift: LayoutPos) {
        if let Some(position) = self.node_positions.get_mut(node_key) {
            *position = GraphLayout::snap_to_grid(*position + shift);
        }
    }

    pub fn shift_note(&mut self, note_key: NoteKey, shift: LayoutPos) {
        if let Some(note) = self.notes.get_mut(note_key) {
            note.position = GraphLayout::snap_to_grid(note.position + shift);
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

    pub fn remove_note(&mut self, id: NoteKey) {
        self.notes.remove(id);
    }

    pub fn update_note_position_unsnapped(&mut self, note_key: NoteKey, new_position: LayoutPos) {
        if let Some(note) = self.notes.get_mut(note_key) {
            note.position = new_position;
        }
    }

    pub fn update_node_position_unsnapped(&mut self, node_key: NodeKey, delta: LayoutPos) {
        if let Some(note) = self.node_positions.get_mut(node_key) {
            note.x += delta.x;
            note.y += delta.y;
        }
    }

    pub fn update_note_content(&mut self, note_key: NoteKey, content: String) {
        if let Some(note) = self.notes.get_mut(note_key) {
            note.content = content;
        }
    }

    pub fn get_note(&self, note_key: NoteKey) -> Option<&Note> {
        self.notes.get(note_key)
    }

    pub fn notes_iter(&self) -> impl Iterator<Item = (NoteKey, &Note)> {
        self.notes.iter()
    }

    pub fn nodes_iter(&self) -> impl Iterator<Item = (NodeKey, &LayoutPos)> {
        self.node_positions.iter()
    }
}
