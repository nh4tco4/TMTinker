use paste::paste;

macro_rules! generate_shortcuts {
    ($( ($func_name:ident, $key:ident) ), +) => {
        $(
            paste! {
            #[inline]
            pub fn $func_name() -> egui::Key {
                egui::Key::$key
            }

            #[inline]
            pub fn [< text_ $func_name >]() -> &'static str {
                egui::Key::$key.name()
            }
            }
        )+
    };
}

generate_shortcuts!(
    (left_node_shortcut, A),
    (right_node_shortcut, D),
    (word_left_node_shortcut, Q),
    (word_right_node_shortcut, E),
    (link_node_shortcut, S),
    (delete_node_shortcut, X),
    (move_node_shortcut, V),
    (space_node_shortcut, W),
    (copy_node_shortcut, C),
    (submachine_node_shortcut, M),
    (start_node_shortcut, T),
    (end_node_shortcut, G),
    (note_node_shorcut, B)
);
