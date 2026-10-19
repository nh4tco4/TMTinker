use std::collections::{HashSet, hash_set::Iter};

use slotmap::new_key_type;

use crate::{core::alphabet::ValidatedAlphabet, diagram::model::submachine::NodeKey};

new_key_type! {
    pub struct EdgeKey;
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub enum TransitionSymbolsError {
    #[error("Symbols must not be empty")]
    Empty,

    #[error("Alphabet doesn't contain selected character.")]
    AlphabetDoesNotContain(char),

    #[error("Alphabet contains duplicated characters.")]
    DuplicatedCharacter(char),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct TransitionSymbols(HashSet<char>);

pub struct TransitionSymbolsIter<'a> {
    inner: Iter<'a, char>,
}

impl<'a> Iterator for TransitionSymbolsIter<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().copied()
    }
}

impl<'a> IntoIterator for &'a TransitionSymbols {
    type Item = char;
    type IntoIter = TransitionSymbolsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl TransitionSymbols {
    pub fn new(
        symbols: impl IntoIterator<Item = char>,
        alphabet: &ValidatedAlphabet,
    ) -> Result<Self, TransitionSymbolsError> {
        let value = Self::new_exclusions(symbols, alphabet)?;
        if value.0.is_empty() {
            return Err(TransitionSymbolsError::Empty);
        }
        Ok(value)
    }

    /// Unlike an inclusion set, an exclusion set may be empty (accept the whole alphabet).
    pub fn new_exclusions(
        symbols: impl IntoIterator<Item = char>,
        alphabet: &ValidatedAlphabet,
    ) -> Result<Self, TransitionSymbolsError> {
        let alphabet_set: HashSet<char> = alphabet.iter().copied().collect();
        let mut symbols_set = HashSet::new();

        for symbol in symbols {
            if !alphabet_set.contains(&symbol) {
                return Err(TransitionSymbolsError::AlphabetDoesNotContain(symbol));
            }

            if !symbols_set.insert(symbol) {
                return Err(TransitionSymbolsError::DuplicatedCharacter(symbol));
            }
        }

        Ok(Self(symbols_set))
    }

    pub fn contains(&self, symbol: char) -> bool {
        self.0.contains(&symbol)
    }

    pub fn iter(&self) -> TransitionSymbolsIter<'_> {
        TransitionSymbolsIter {
            inner: self.0.iter(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Debug)]
pub struct Edge {
    chars: TransitionSymbols,
    #[serde(default)]
    inverse: bool,
    source: NodeKey,
    target: NodeKey,
}

impl Edge {
    pub fn accepts(&self, symbol: char, alphabet: &ValidatedAlphabet) -> bool {
        alphabet.contains(symbol) && (self.chars.contains(symbol) != self.inverse)
    }

    pub fn accepts_all(&self, alphabet: &ValidatedAlphabet) -> bool {
        alphabet
            .iter()
            .copied()
            .all(|symbol| self.accepts(symbol, alphabet))
    }

    pub fn new(chars: TransitionSymbols, source: NodeKey, target: NodeKey) -> Edge {
        Self {
            chars,
            inverse: false,
            source,
            target,
        }
    }

    pub fn symbols(&self) -> &TransitionSymbols {
        &self.chars
    }

    pub fn inverse(&self) -> bool {
        self.inverse
    }

    pub fn set_inverse(&mut self, inverse: bool) {
        self.inverse = inverse;
    }

    pub fn set_symbols(&mut self, symbols: TransitionSymbols) {
        self.chars = symbols;
    }

    pub fn source(&self) -> NodeKey {
        self.source
    }

    pub fn target(&self) -> NodeKey {
        self.target
    }
}
