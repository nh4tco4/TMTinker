use std::collections::HashMap;
use std::ops::RangeInclusive;

pub const SPACE: char = ' ';
pub const LAMBDA: char = 'λ';

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Tape {
    items: HashMap<i32, char>,
}

impl Tape {
    /// Creates an empty tape.
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    fn reset_tape(&mut self) {
        self.items.clear();
    }

    /// Creates a tape from an iterator of chars, starting at index 0.
    pub fn from_iter(chars: impl IntoIterator<Item = char>) -> Self {
        let items = chars
            .into_iter()
            .enumerate()
            .map(|(i, c)| (i as i32, c))
            .collect();
        Self { items }
    }

    /// Returns `Some(char)` if the cell exists, `None` otherwise.
    #[allow(dead_code)]
    pub fn peek(&self, index: i32) -> Option<char> {
        self.items.get(&index).copied()
    }

    /// Inserts SPACE at `index` if not already present.
    pub fn extend_tape(&mut self, index: i32) {
        self.items.entry(index).or_insert(SPACE);
    }

    /// Writes `value` at `index`.
    pub fn write(&mut self, value: char, index: i32) {
        self.items.insert(index, value);
    }

    /// Reads cell; returns `SPACE` if cell is absent.
    pub fn read_or_space(&self, index: i32) -> char {
        self.items.get(&index).copied().unwrap_or(SPACE)
    }

    /// Reads cell; returns `LAMBDA` if cell is absent or contains SPACE.
    pub fn read_or_lambda(&self, index: i32) -> char {
        match self.items.get(&index).copied() {
            Some(SPACE) | None => LAMBDA,
            Some(c) => c,
        }
    }

    /// Read-only access to the underlying map (needed by UI for rendering).
    pub fn items(&self) -> &HashMap<i32, char> {
        &self.items
    }

    /// Formats a range of tape cells into a human-readable string.
    ///
    /// - `space_to_lambda` — replace SPACE with λ for display
    /// - `head_position`   — if `Some(pos)`, wraps that cell in parentheses
    ///
    /// Example: `[ λ, (0), 1, λ >`
    pub fn format_tape(
        &self,
        range: RangeInclusive<i32>,
        space_to_lambda: bool,
        head_position: Option<i32>,
    ) -> String {
        let mut result = String::from("[ ");

        for (i, pos) in range.enumerate() {
            if i > 0 {
                result.push_str(", ");
            }

            let ch = self.read_or_lambda(pos);
            let display_ch = if space_to_lambda && ch == SPACE {
                LAMBDA
            } else {
                ch
            };

            if head_position == Some(pos) {
                result.push('(');
                result.push(display_ch);
                result.push(')');
            } else {
                result.push(display_ch);
            }
        }

        result.push_str(" >");
        result
    }
}

impl Default for Tape {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Traits

pub trait BasicCommands {
    fn read(&self) -> char;
    fn read_index(&self, index: i32) -> char;
    fn write(&mut self, value: char);
    fn left(&mut self, power: u32);
    fn right(&mut self, power: u32);
    fn word_left(&mut self, power: u32);
    fn word_right(&mut self, power: u32);
    fn copy(&mut self, power: u32);
    fn format_tape(
        &self,
        range: RangeInclusive<i32>,
        space_to_lambda: bool,
        include_head: bool,
    ) -> String;
    #[allow(dead_code)]
    fn print_tape(&self, range: RangeInclusive<i32>, space_to_lambda: bool, include_head: bool);
}

pub trait AdvancedCommands: BasicCommands {
    fn delete_word_left(&mut self);
    fn delete_word_right(&mut self);
    fn skip_sequence_left(&mut self, power: u32);
    fn skip_sequence_right(&mut self, power: u32);
    #[allow(dead_code)]
    fn inverse_word(&mut self);
    fn shift_one_word_backwards(&mut self);
    fn copy_nth_word(&mut self, n: u32);
}

// ─── TMCore

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TMCore {
    tape: Tape,
    head_position: i32,
    alphabet: Vec<char>,
}

impl TMCore {
    /// Creates a machine with the given alphabet and an empty tape.
    pub fn new(alphabet: Vec<char>) -> Self {
        Self {
            tape: Tape::new(),
            head_position: 0,
            alphabet,
        }
    }

    /// Creates a machine from a string that will be written to the tape
    /// starting at index 0.
    #[allow(dead_code)]
    pub fn with_tape(tape_str: &str, head_position: i32, alphabet: Vec<char>) -> Self {
        Self {
            tape: Tape::from_iter(tape_str.chars()),
            head_position,
            alphabet,
        }
    }

    pub fn head_position(&self) -> i32 {
        self.head_position
    }

    #[allow(dead_code)]
    pub fn alphabet(&self) -> &[char] {
        &self.alphabet
    }

    /// Read-only view into the tape cells (used by UI for rendering).
    pub fn tape_items(&self) -> &HashMap<i32, char> {
        self.tape.items()
    }

    /// Resets the tape to an empty state.
    pub fn reset_tape(&mut self) {
        self.tape.reset_tape();
    }

    /// Writes a character directly at an arbitrary tape index,
    /// bypassing the alphabet check. Intended for UI tape editing.
    pub fn write_at(&mut self, index: i32, ch: char) {
        self.tape.write(ch, index);
    }

    /// Moves the head directly to `pos` without stepping through cells.
    /// Extends the tape at the destination if needed.
    pub fn seek(&mut self, pos: i32) {
        self.tape.extend_tape(pos);
        self.head_position = pos;
    }
}

impl Default for TMCore {
    fn default() -> Self {
        Self {
            tape: Tape::from_iter(" 01 ".chars()),
            head_position: 0,
            alphabet: vec![SPACE, '0', '1'],
        }
    }
}

// ─── BasicCommands

impl BasicCommands for TMCore {
    /// Returns the character under the head.
    ///
    /// Returns SPACE if the cell has never been written to.
    fn read(&self) -> char {
        self.tape.read_or_space(self.head_position)
    }

    /// Returns the character at an arbitrary index.
    ///
    /// Returns SPACE if the cell has never been written to.
    fn read_index(&self, index: i32) -> char {
        self.tape.read_or_space(index)
    }

    /// Writes `value` under the head.
    ///
    /// Does nothing if `value` is not in the alphabet.
    fn write(&mut self, value: char) {
        if self.alphabet.contains(&value) {
            self.tape.write(value, self.head_position);
        }
    }

    /// Moves the head `power` cells to the left.
    ///
    /// ```text
    /// [λ, λ, (1), λ > -left(1)-> [λ, (λ), 1, λ >
    /// ```
    fn left(&mut self, power: u32) {
        for _ in 0..power {
            // Extend the tape at the destination before moving.
            let dest = self.head_position - 1;
            self.tape.extend_tape(dest);
            self.head_position = dest;
        }
    }

    /// Moves the head `power` cells to the right.
    ///
    /// ```text
    /// [λ, (0), 1, λ > -right(1)-> [λ, 0, (1), λ >
    /// ```
    fn right(&mut self, power: u32) {
        for _ in 0..power {
            let dest = self.head_position + 1;
            self.tape.extend_tape(dest);
            self.head_position = dest;
        }
    }

    /// Moves the head `power` words to the left,
    /// stopping on the SPACE before the word.
    ///
    /// ```text
    /// [λ, 1, 1, (λ) > -word_left(1)-> [(λ), 1, 1, λ >
    /// ```
    fn word_left(&mut self, power: u32) {
        for _ in 0..power {
            loop {
                self.left(1);
                if self.read() == SPACE {
                    break;
                }
            }
        }
    }

    /// Moves the head `power` words to the right,
    /// stopping on the SPACE after the word.
    ///
    /// ```text
    /// [(λ), 1, 1, λ > -word_right(1)-> [λ, 1, 1, (λ) >
    /// ```
    fn word_right(&mut self, power: u32) {
        for _ in 0..power {
            loop {
                self.right(1);
                if self.read() == SPACE {
                    break;
                }
            }
        }
    }

    /// Copies `power` words from the left of the head and appends them
    /// to the right, preserving order.
    ///
    /// ```text
    /// [ λ, v1, v2, (λ) > -copy(2)-> [ λ, v1, v2, λ, v1, v2, (λ) >
    /// ```
    fn copy(&mut self, power: u32) {
        let mut words: Vec<Vec<char>> = Vec::with_capacity(power as usize);

        // Collect `power` words going left; each word's chars arrive reversed.
        for _ in 0..power {
            self.left(1);
            let mut word = Vec::new();
            while self.read() != SPACE {
                word.push(self.read());
                self.left(1);
            }
            // Restore char order within the word.
            word.reverse();
            words.push(word);
        }

        // `words[0]` is the word closest to the original head position,
        // `words[1]` is one further left, etc. — reverse so we write them
        // left-to-right in the original order.
        words.reverse();

        // Jump past `power` words to the right to find the insertion point.
        self.word_right(power + 1);

        for word in words {
            for ch in word {
                self.write(ch);
                self.right(1);
            }
            self.right(1); // separator SPACE
        }

        // Step back onto the last SPACE (head rests on separator after last word).
        self.left(1);
    }

    fn format_tape(
        &self,
        range: RangeInclusive<i32>,
        space_to_lambda: bool,
        include_head: bool,
    ) -> String {
        let head = include_head.then_some(self.head_position);
        self.tape.format_tape(range, space_to_lambda, head)
    }

    fn print_tape(&self, range: RangeInclusive<i32>, space_to_lambda: bool, include_head: bool) {
        println!("{}", self.format_tape(range, space_to_lambda, include_head));
    }
}

// ─── AdvancedCommands

impl AdvancedCommands for TMCore {
    /// Deletes the word immediately to the left of the head.
    ///
    /// ```text
    /// [ λ, v1, (λ) > -delete_word_left-> [ (λ), λλ, λ >
    /// ```
    fn delete_word_left(&mut self) {
        self.left(1);
        while self.read() != SPACE {
            self.write(SPACE);
            self.left(1);
        }
    }

    /// Deletes the word immediately to the right of the head.
    ///
    /// ```text
    /// [ λ, (λ), v1, λ > -delete_word_right-> [ λ, λλ, λ, (λ) >
    /// ```
    fn delete_word_right(&mut self) {
        self.right(1);
        while self.read() != SPACE {
            self.write(SPACE);
            self.right(1);
        }
    }

    /// Skips `power` groups of words to the left.
    ///
    /// ```text
    /// [ λ, v1, λ, v3, (λ) > -skip_sequence_left(1)-> [ (λ), v1, λ, v3, λ >
    /// ```
    fn skip_sequence_left(&mut self, power: u32) {
        for _ in 0..power {
            while self.read_index(self.head_position - 1) != SPACE {
                self.word_left(1);
            }
        }
    }

    /// Skips `power` groups of words to the right.
    ///
    /// ```text
    /// [ (λ), v1, λ, v3, λ > -skip_sequence_right(1)-> [ λ, v1, λ, v3, (λ) >
    /// ```
    fn skip_sequence_right(&mut self, power: u32) {
        for _ in 0..power {
            while self.read_index(self.head_position + 1) != SPACE {
                self.word_right(1);
            }
        }
    }

    /// Reverses the word immediately to the left of the head.
    ///
    /// ```text
    /// [λ, 1, 0, (λ) > -inverse_word-> [(λ), 0, 1, λ >
    /// ```
    fn inverse_word(&mut self) {
        self.left(1);

        let mid = self.head_position;

        let mut left = mid;
        while self.read_index(left) != SPACE {
            left -= 1;
        }
        left += 1;

        let mut right = mid;
        while self.read_index(right) != SPACE {
            right += 1;
        }
        right -= 1;

        while left < right {
            let lc = self.read_index(left);
            let rc = self.read_index(right);
            self.tape.write(rc, left);
            self.tape.write(lc, right);
            left += 1;
            right -= 1;
        }

        self.word_left(1);
    }

    /// Shifts the word under the head one SPACE closer to the previous word.
    ///
    /// ```text
    /// [λ, v1, λ, λ, v2, (λ) > -shift_one_word_backwards-> [ λ, v1, λ, v2, (λ) >
    /// ```
    ///
    /// If there is only one word on the tape this will loop forever.
    fn shift_one_word_backwards(&mut self) {
        let mut word = Vec::new();

        self.left(1);
        while self.read() != SPACE {
            word.push(self.read());
            self.write(SPACE);
            self.left(1);
        }

        while self.read_index(self.head_position - 1) == SPACE {
            self.left(1);
        }
        self.right(1);

        for ch in word {
            self.write(ch);
            self.right(1);
        }

        self.right(1);
    }

    /// Copies the n-th word from the left and appends it after the head.
    ///
    /// ```text
    /// [ λ, v1, λ, v2, λ, v3, (λ) > -copy_nth_word(2)->
    ///   [ λ, v1, λ, v2, λ, v3, λ, v2, (λ) >
    /// ```
    fn copy_nth_word(&mut self, n: u32) {
        let saved = self.head_position;

        // Walk left n words to read the target word.
        self.word_left(n);
        self.left(1);

        let mut word = Vec::new();
        while self.read() != SPACE {
            word.push(self.read());
            self.left(1);
        }
        word.reverse();

        // Return to the original position and write.
        self.head_position = saved;
        self.right(1);

        for ch in word {
            self.write(ch);
            self.right(1);
        }

        self.right(1);
    }
}

// ─── Tests

#[cfg(test)]
mod tests {
    use super::*;

    fn fmt(tm: &TMCore, end: i32) -> String {
        tm.format_tape(0..=end, true, true)
    }

    // --- left

    #[test]
    fn test_left_1() {
        // default tape: " 01 ", head starts at 0
        // after left(1): head is at -1 (outside 0..=3)
        let mut tm = TMCore::default();
        tm.left(1);
        assert_eq!(tm.head_position(), -1);
        assert_eq!(tm.format_tape(-1..=3, true, true), "[ (λ), λ, 0, 1, λ >");
    }

    #[test]
    fn test_left_from_index_4() {
        let mut tm = TMCore {
            head_position: 4,
            ..Default::default()
        };
        tm.left(1);
        assert_eq!(fmt(&tm, 3), "[ λ, 0, 1, (λ) >");
    }

    #[test]
    fn test_left_3_from_index_4() {
        let mut tm = TMCore {
            head_position: 4,
            ..Default::default()
        };
        tm.left(3);
        assert_eq!(fmt(&tm, 3), "[ λ, (0), 1, λ >");
    }

    // --- right

    #[test]
    fn test_right_1() {
        let mut tm = TMCore::default();
        tm.right(1);
        assert_eq!(fmt(&tm, 3), "[ λ, (0), 1, λ >");
    }

    #[test]
    fn test_right_3() {
        let mut tm = TMCore::default();
        tm.right(3);
        assert_eq!(fmt(&tm, 3), "[ λ, 0, 1, (λ) >");
    }

    #[test]
    fn test_right_4() {
        let mut tm = TMCore::default();
        tm.right(4);
        assert_eq!(fmt(&tm, 4), "[ λ, 0, 1, λ, (λ) >");
    }

    // --- word_left

    #[test]
    fn test_word_left_from_end() {
        let mut tm = TMCore {
            head_position: 3,
            ..Default::default()
        };
        tm.word_left(1);
        assert_eq!(fmt(&tm, 3), "[ (λ), 0, 1, λ >");
    }

    #[test]
    fn test_word_left_2_from_index_4() {
        let mut tm = TMCore {
            head_position: 4,
            ..Default::default()
        };
        tm.word_left(2);
        assert_eq!(fmt(&tm, 3), "[ (λ), 0, 1, λ >");
    }

    #[test]
    fn test_word_left_mid_word() {
        let mut tm = TMCore {
            head_position: 2,
            ..Default::default()
        };
        tm.word_left(1);
        assert_eq!(fmt(&tm, 3), "[ (λ), 0, 1, λ >");
    }

    // --- word_right

    #[test]
    fn test_word_right_1() {
        let mut tm = TMCore::default();
        tm.word_right(1);
        assert_eq!(fmt(&tm, 3), "[ λ, 0, 1, (λ) >");
    }

    #[test]
    fn test_word_right_2_from_minus_1() {
        let mut tm = TMCore {
            head_position: -1,
            ..Default::default()
        };
        tm.word_right(2);
        assert_eq!(fmt(&tm, 3), "[ λ, 0, 1, (λ) >");
    }

    // --- write

    #[test]
    fn test_write_1() {
        let mut tm = TMCore::default();
        tm.write('1');
        assert_eq!(fmt(&tm, 3), "[ (1), 0, 1, λ >");
    }

    #[test]
    fn test_write_space() {
        let mut tm = TMCore::default();
        tm.right(2);
        tm.write(SPACE);
        assert_eq!(fmt(&tm, 3), "[ λ, 0, (λ), λ >");
    }

    // --- inverse_word

    #[test]
    fn test_inverse_word() {
        let mut tm = TMCore {
            head_position: 3,
            ..Default::default()
        };
        tm.inverse_word();
        assert_eq!(fmt(&tm, 3), "[ (λ), 1, 0, λ >");
    }

    // --- copy

    #[test]
    fn test_copy_single_word() {
        // tape: "  101  ", head at index 5 (first SPACE after 101)
        let mut tm = TMCore::with_tape("  101  ", 5, vec![SPACE, '0', '1']);
        tm.copy(1);
        let expected = "[ λ, λ, 1, 0, 1, λ, 1, 0, 1, (λ) >";
        assert_eq!(tm.format_tape(0..=9, true, true), expected);
    }

    #[test]
    fn test_copy_two_words() {
        // tape: " 10 01 ", head at index 6 (SPACE after "01")
        let mut tm = TMCore::with_tape(" 10 01 ", 6, vec![SPACE, '1', '0']);
        tm.copy(2);
        assert_eq!(
            tm.format_tape(0..=13, true, true),
            "[ λ, 1, 0, λ, 0, 1, λ, 1, 0, λ, 0, 1, (λ), λ >"
        );
    }
}
