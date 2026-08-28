// use crate::{
//     app::App,
//     core::graph::{Edge, EdgeKey, Node, NodeKey},
//     editor::layout::LayoutPos,
// };
// use slotmap::SlotMap;
// use std::collections::HashMap;
//
// #[derive(Clone)]
// pub struct EditorMemento {
//     pub node_positions: HashMap<NodeKey, LayoutPos>,
//     pub nodes: SlotMap<NodeKey, Node>,
//     pub edges: SlotMap<EdgeKey, Edge>,
// }
//
// impl EditorMemento {
//     pub fn from_app(app: &App) -> Self {
//         let (graph, editor) = app.current_graph_and_editor();
//         Self {
//             node_positions: editor.node_positions.clone(),
//             nodes: graph.nodes.clone(),
//             edges: graph.edges.clone(),
//         }
//     }
//
//     pub fn apply_to(&self, app: &mut App) {
//         let graph = app.current_graph_mut();
//         graph.nodes.clone_from(&self.nodes);
//         graph.edges.clone_from(&self.edges);
//
//         let editor = app.current_editor_mut();
//         editor.node_positions.clone_from(&self.node_positions);
//         app.dragged_nodes.clear();
//     }
// }
//
// #[derive(Default)]
// pub struct UndoManager {
//     history: Vec<EditorMemento>,
//     current: usize,
//     max_size: usize,
// }
//
// impl UndoManager {
//     pub fn new(max_size: usize) -> Self {
//         Self {
//             history: Vec::with_capacity(max_size),
//             current: 0,
//             max_size,
//         }
//     }
//
//     pub fn save_checkpoint(app: &mut App) {
//         let memento = EditorMemento::from_app(app);
//         let um = &mut app.undo_manager;
//
//         um.history.truncate(um.current + 1);
//
//         um.history.push(memento);
//         um.current = um.history.len().saturating_sub(1);
//
//         if um.history.len() > um.max_size {
//             let excess = um.history.len() - um.max_size;
//             um.history.drain(0..excess);
//             um.current = um.current.saturating_sub(excess);
//         }
//     }
//
//     pub fn undo(app: &mut App) -> bool {
//         let um = &mut app.undo_manager;
//
//         if !Self::can_undo_static(um) {
//             return false;
//         }
//
//         um.current -= 1;
//         Self::restore_current(app);
//         true
//     }
//
//     pub fn redo(app: &mut App) -> bool {
//         let um = &mut app.undo_manager;
//
//         if !Self::can_redo_static(um) {
//             return false;
//         }
//
//         um.current += 1;
//         Self::restore_current(app);
//         true
//     }
//
//     pub fn can_undo(app: &App) -> bool {
//         Self::can_undo_static(&app.undo_manager)
//     }
//
//     pub fn can_redo(app: &App) -> bool {
//         Self::can_redo_static(&app.undo_manager)
//     }
//
//     pub fn clear(app: &mut App) {
//         let um = &mut app.undo_manager;
//         um.history.clear();
//         um.current = 0;
//     }
//
//     pub fn current_index(app: &App) -> usize {
//         app.undo_manager.current
//     }
//
//     pub fn history_len(app: &App) -> usize {
//         app.undo_manager.history.len()
//     }
//
//     fn restore_current(app: &mut App) {
//         let memento = {
//             let um = &app.undo_manager;
//             um.history.get(um.current).cloned()
//         }
//         .expect("History should not be empty when restore_current is called");
//
//         memento.apply_to(app);
//     }
//
//     fn can_undo_static(um: &UndoManager) -> bool {
//         um.current > 0 && !um.history.is_empty()
//     }
//
//     fn can_redo_static(um: &UndoManager) -> bool {
//         um.current + 1 < um.history.len()
//     }
// }
