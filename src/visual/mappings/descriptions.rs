use paste::paste;

macro_rules! generate_desriptions {
    ($( ($func_name:ident, $desc:tt) ), +) => {
        $(
            paste! {
            #[inline]
            pub fn [< $func_name _description >]() -> &'static str {
                $desc
            }
            }
        )+
    };
}

generate_desriptions!(
    (left_node, "Moves head power cells to the left."),
    (right_node, "Moves head power cells to the right."),
    (word_left_node, "Moves head power words to the left."),
    (word_right_node, "Moves head power words to the right."),
    (space_node, "Places selected sign under the working head"),
    // TODO make a proper description
    (submachine_node, "..."),
    (
        copy_node,
        "Copies power words from the left of the working head to its right, preserving words order."
    ),
    (start_node, "Entry point of each submachine."),
    (end_node, "Exit point of each submachine.")
);
