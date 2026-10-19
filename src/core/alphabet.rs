use std::{collections::HashSet, sync::LazyLock};

static ALLOWED_CHARACTERS: LazyLock<HashSet<char>> = LazyLock::new(|| {
    [
        ' ', '\\', '(', ')', '|', '!', '@', '#', '%', '^', '&', '*', '-', '+', '=', '_', '/', '?',
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H',
        'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ]
    .into_iter()
    .collect()
});

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ValidatedAlphabet {
    characters: Vec<char>,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum AlphabetCreationError {
    #[error("Selected character not allowed: {0}")]
    CharacterNotAllowed(char),
    #[error("Every alphabet must include lambda (space) character.")]
    AlphabetDoesNotHaveLambda,
    #[error(
        "Every alphabet should consist of at least two characters, one of which has to be lambda (space)."
    )]
    AlphabetTooSmall,
    #[error("Alphabet has duplicated character: {0}")]
    DuplicatedCharacter(char),
}

impl ValidatedAlphabet {
    pub fn new(alphabet: impl IntoIterator<Item = char>) -> Result<Self, AlphabetCreationError> {
        let characters: Vec<char> = alphabet.into_iter().collect();
        if characters.len() < 2 {
            return Err(AlphabetCreationError::AlphabetTooSmall);
        }

        let mut unique = HashSet::with_capacity(characters.len());
        for &character in &characters {
            if !ALLOWED_CHARACTERS.contains(&character) {
                return Err(AlphabetCreationError::CharacterNotAllowed(character));
            }
            if !unique.insert(character) {
                return Err(AlphabetCreationError::DuplicatedCharacter(character));
            }
        }

        if !unique.contains(&' ') {
            return Err(AlphabetCreationError::AlphabetDoesNotHaveLambda);
        }

        Ok(Self { characters })
    }

    pub fn contains(&self, character: char) -> bool {
        self.characters.contains(&character)
    }

    pub fn len(&self) -> usize {
        self.characters.len()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &char> {
        self.characters.iter()
    }

    pub fn clone_iter(&self) -> impl ExactSizeIterator<Item = char> + '_ {
        self.characters.iter().copied()
    }
}
