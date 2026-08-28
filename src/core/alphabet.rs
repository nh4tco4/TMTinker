use std::collections::HashSet;
use std::sync::LazyLock;

static ALLOWED_CHARACTERS: LazyLock<HashSet<char>> = LazyLock::new(|| {
    [
        ' ', '\\', '(', ')', '|', '!', '@', '#', '%', '^', '&', '*', '-', '+', '=', '_', '/', '?',
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '0',
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'
    ].into_iter().collect()
});

#[derive(Debug, Clone)]
pub struct ValidatedAlphabet {
    characters: Vec<char>
}

#[derive(thiserror::Error, Debug)]
pub enum AlphabetCreationError {
    #[error("Selected character not allowed.")]
    CharacterNotAllowed,

    #[error("Every alphabet must include lambda (space) character.")]
    AlphabetDoesNotHaveLambda,

    #[error("Every alphabet should consist of at least two character, one of which has to be lambda (space).")]
    AlphabetTooSmall,

    #[error("Alphabet has duplicated characters.")]
    DuplicatedCharacters,
}

impl ValidatedAlphabet {
    pub fn new(alphabet: impl IntoIterator<Item = char>) -> Result<Self, AlphabetCreationError> {
        let chars: Vec<char> = alphabet.into_iter().collect();
        let total_len = chars.len();

        if total_len < 2 {
            return Err(AlphabetCreationError::AlphabetTooSmall);
        }

        let mut unique_chars = HashSet::with_capacity(total_len);
        let mut has_space = false;

        for &c in &chars {
            if c == ' ' {
                has_space = true;
            }

            if !ALLOWED_CHARACTERS.contains(&c) {
                return Err(AlphabetCreationError::CharacterNotAllowed);
            }

            if !unique_chars.insert(c) {
                return Err(AlphabetCreationError::DuplicatedCharacters);
            }
        }

        if !has_space {
            return Err(AlphabetCreationError::AlphabetDoesNotHaveLambda);
        }

        Ok(Self { characters: chars })
    }

    pub fn clone_iter(&self) -> impl Iterator<Item = char> {
        self.characters.clone().into_iter()
    }

    pub fn iter(&self) -> impl Iterator<Item = &char> {
        self.characters.iter()
    }
}
