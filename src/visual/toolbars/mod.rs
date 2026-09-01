pub mod control_window;
pub mod menu;
pub mod node_control_windows;
pub mod tape_window;
pub mod tools_window;
pub mod edge_control_window;

use crate::diagram::model::submachine::SubmachineKey;

pub enum Page {
    Tinkering(SubmachineKey),
    Menu,
}
