use crate::ui::theme::{LAMBDA, SPACE};

pub fn formatted_buffer(
    alphabet: &crate::core::alphabet::ValidatedAlphabet,
    buffer: &str,
) -> String {
    buffer
        .chars()
        .filter_map(|c| {
            if alphabet.contains(c) || c == LAMBDA {
                Some(format_char(c))
            } else {
                None
            }
        })
        .collect()
}

fn format_char(c: char) -> char {
    if c == SPACE { LAMBDA } else { c }
}

pub fn buffer_to_load(buffer: &str) -> String {
    buffer
        .chars()
        .map(|c| if c == LAMBDA { SPACE } else { c })
        .collect()
}
