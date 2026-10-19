use std::collections::HashMap;

use crate::core::alphabet::ValidatedAlphabet;

pub type TapeIndex = i32;
pub const BLANK: char = ' ';

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Tape {
    cells: HashMap<TapeIndex, char>,
    head_position: TapeIndex,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum TapeCreationError {
    #[error("Character is not in alphabet: {0}")]
    CharacterNotInAlphabet(char),
    #[error("No opening head brace.")]
    NoOpeningHeadBrace,
    #[error("No closing head brace.")]
    NoClosingHeadBrace,
    #[error("Found multiple tape heads.")]
    MultipleHeads,
    #[error("Head position was not found on the tape.")]
    MissingHead,
    #[error("A tape head must contain exactly one symbol.")]
    InvalidHead,
}

impl Tape {
    pub fn set_cell(&mut self, index: TapeIndex, symbol: char) {
        if symbol == BLANK {
            self.cells.remove(&index);
        } else {
            self.cells.insert(index, symbol);
        }
    }

    pub fn set_head(&mut self, index: TapeIndex) {
        self.head_position = index;
    }

    pub fn validate(&self, alphabet: &ValidatedAlphabet) -> Result<(), String> {
        if self
            .cells
            .values()
            .any(|c| !alphabet.contains(*c) || *c == BLANK)
        {
            return Err("Tape contains an invalid symbol or a noncanonical blank cell".into());
        }
        Ok(())
    }

    pub fn from_text(
        text: &str,
        origin: TapeIndex,
        alphabet: &ValidatedAlphabet,
    ) -> Result<Self, String> {
        let mut tape = Self::from_tape(text, alphabet).map_err(|e| e.to_string())?;
        tape.head_position = tape
            .head_position
            .checked_add(origin)
            .ok_or("Tape index overflow")?;
        tape.cells = tape
            .cells
            .into_iter()
            .map(|(i, c)| {
                i.checked_add(origin)
                    .map(|index| (index, c))
                    .ok_or_else(|| "Tape index overflow".to_owned())
            })
            .collect::<Result<_, _>>()?;
        Ok(tape)
    }

    pub fn to_text(&self) -> (String, TapeIndex) {
        let min = self
            .cells
            .keys()
            .copied()
            .min()
            .unwrap_or(self.head_position)
            .min(self.head_position);
        let max = self
            .cells
            .keys()
            .copied()
            .max()
            .unwrap_or(self.head_position)
            .max(self.head_position);

        if i64::from(max) - i64::from(min) > 4096 {
            return (String::new(), min);
        }
        let mut text = String::new();
        for index in min..=max {
            if index == self.head_position {
                text.push('(');
            }
            text.push(if self.cell(index) == BLANK {
                'λ'
            } else {
                self.cell(index)
            });
            if index == self.head_position {
                text.push(')');
            }
        }
        (text, min)
    }
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_tape(
        tape_string: impl AsRef<str>,
        alphabet: &ValidatedAlphabet,
    ) -> Result<Self, TapeCreationError> {
        let mut cells = HashMap::new();
        let mut head_position = None;
        let mut current_index = 0;
        let mut inside_head = false;
        let mut head_symbols = 0;

        for raw in tape_string.as_ref().chars() {
            match raw {
                '(' => {
                    if inside_head || head_position.is_some() {
                        return Err(TapeCreationError::MultipleHeads);
                    }
                    inside_head = true;
                    head_position = Some(current_index);
                }
                ')' => {
                    if !inside_head {
                        return Err(TapeCreationError::NoOpeningHeadBrace);
                    }
                    if head_symbols != 1 {
                        return Err(TapeCreationError::InvalidHead);
                    }
                    inside_head = false;
                }
                '[' | ']' | '>' => {}
                symbol => {
                    let symbol = if symbol == 'λ' { BLANK } else { symbol };
                    if !alphabet.contains(symbol) {
                        return Err(TapeCreationError::CharacterNotInAlphabet(symbol));
                    }
                    if symbol != BLANK {
                        cells.insert(current_index, symbol);
                    }
                    current_index += 1;
                    if inside_head {
                        head_symbols += 1;
                    }
                }
            }
        }

        if inside_head {
            return Err(TapeCreationError::NoClosingHeadBrace);
        }

        Ok(Self {
            cells,
            head_position: head_position.ok_or(TapeCreationError::MissingHead)?,
        })
    }

    pub fn matches_output(&self, expected: &Self) -> bool {
        if self.cells.len() != expected.cells.len() {
            return false;
        }
        let shift = i64::from(expected.head_position) - i64::from(self.head_position);
        self.cells.iter().all(|(&index, symbol)| {
            i32::try_from(i64::from(index) + shift)
                .ok()
                .and_then(|index| expected.cells.get(&index))
                == Some(symbol)
        })
    }

    pub fn read(&self) -> char {
        self.cell(self.head_position)
    }

    pub fn cell(&self, index: TapeIndex) -> char {
        self.cells.get(&index).copied().unwrap_or(BLANK)
    }

    pub fn head_position(&self) -> TapeIndex {
        self.head_position
    }

    pub fn move_right(&mut self) {
        self.head_position = self.head_position.saturating_add(1);
    }

    pub fn move_left(&mut self) {
        self.head_position = self.head_position.saturating_sub(1);
    }

    pub fn write(&mut self, symbol: char) {
        if symbol == BLANK {
            self.cells.remove(&self.head_position);
        } else {
            self.cells.insert(self.head_position, symbol);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_moves_tape() {
        let alphabet = ValidatedAlphabet::new([' ', '0', '1'])
            .expect("the test alphabet contains blank and unique allowed characters");
        let mut tape = Tape::from_tape("0(1)λ", &alphabet)
            .expect("the test tape has exactly one head and valid symbols");
        assert_eq!(tape.read(), '1');
        tape.move_right();
        assert_eq!(tape.read(), BLANK);
        tape.write('0');
        assert_eq!(tape.read(), '0');
    }
}
