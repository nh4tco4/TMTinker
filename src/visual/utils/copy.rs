// use egui::Context;
//
// use crate::app::App;
// use crate::core::graph::{Action, NodeKey};
// use crate::editor::layout::{EditorState, LayoutPos, SelectableId};
//
// #[derive(Debug, Clone)]
// pub struct CopyBuf {
//     pub buf: Vec<SelectableId>,
//     pub reference_point: LayoutPos,
// }
//
// impl Default for CopyBuf {
//     fn default() -> Self {
//         Self {
//             buf: Vec::new(),
//             reference_point: LayoutPos { x: 0.0, y: 0.0 },
//         }
//     }
// }
//
// pub fn paste_copy_buf(app: &mut App, pointer_screen_pos: egui::Pos2) {
//     let copied = &app.copied_buf.clone();
//     if copied.buf.is_empty() {
//         return;
//     }
//
//     let editor = app.current_editor();
//     let pointer_world = editor.screen_to_world(pointer_screen_pos.into());
//
//     let delta = LayoutPos {
//         x: pointer_world.x - copied.reference_point.x,
//         y: pointer_world.y - copied.reference_point.y,
//     };
//
//     let mut old_to_new: std::collections::HashMap<NodeKey, NodeKey> = Default::default();
//
//     for id in &copied.buf {
//         if let SelectableId::Node(old_key) = *id {
//             if let Some(node) = app.current_graph().get_node(old_key) {
//                 let new_key = place_node(app, delta, node.action.clone(), old_key);
//                 old_to_new.insert(old_key, new_key);
//             }
//         }
//     }
//
//     let graph = app.current_graph();
//     let edges_to_copy: Vec<_> = graph
//         .edges
//         .iter()
//         .filter(|(_, edge)| {
//             old_to_new.contains_key(&edge.source) && old_to_new.contains_key(&edge.target)
//         })
//         .map(|(_, edge)| (edge.source, edge.target, edge.chars.clone()))
//         .collect();
//
//     let graph_mut = app.current_graph_mut();
//     for (src, tgt, chars) in edges_to_copy {
//         let new_src = old_to_new[&src];
//         let new_tgt = old_to_new[&tgt];
//         graph_mut.add_edge(chars, new_src, new_tgt);
//     }
//
//     let editor = app.current_editor_mut();
//     editor.selected.clear();
//     for &new_key in old_to_new.values() {
//         editor.selected.selected.push(SelectableId::Node(new_key));
//     }
// }
//
// fn place_node(app: &mut App, delta: LayoutPos, action: Action, node_key: NodeKey) -> NodeKey {
//     let key = app.current_graph_mut().add_node(action);
//     let new_pos = delta
//         + app
//             .current_editor()
//             .get_node_position(&node_key)
//             .unwrap_or(LayoutPos { x: 0.0, y: 0.0 });
//     app.current_editor_mut().set_node_position(key, new_pos);
//     key
// }
//
// pub fn copy_to_buf(app: &mut App, ctx: Context) {
//     let editor = app.current_editor();
//     let selected = editor.selected.selected.clone();
//
//     let reference_point = get_center_of_selection(editor, &selected).unwrap_or_else(|| {
//         ctx.input(|i| i.pointer.hover_pos())
//             .map(|p| editor.screen_to_world(p.into()))
//             .unwrap_or_default()
//     });
//
//     app.copied_buf = CopyBuf {
//         buf: selected,
//         reference_point,
//     };
// }
//
// fn get_center_of_selection(editor: &EditorState, selected: &[SelectableId]) -> Option<LayoutPos> {
//     let mut sum_x = 0.0;
//     let mut sum_y = 0.0;
//     let mut count = 0;
//
//     for id in selected {
//         if let SelectableId::Node(key) = id {
//             if let Some(pos) = editor.get_node_position(key) {
//                 sum_x += pos.x;
//                 sum_y += pos.y;
//                 count += 1;
//             }
//         }
//     }
//
//     if count == 0 {
//         None
//     } else {
//         Some(LayoutPos {
//             x: sum_x / count as f32,
//             y: sum_y / count as f32,
//         })
//     }
// }
