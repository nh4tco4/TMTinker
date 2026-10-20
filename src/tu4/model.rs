use std::sync::Arc;

use crate::core::alphabet::ValidatedAlphabet;

#[derive(Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Tu4Document {
    pub alphabet: ValidatedAlphabet,
    pub source: Arc<str>,
}

impl Tu4Document {
    pub fn new(alphabet: ValidatedAlphabet) -> Self {
        Self {
            alphabet,
            source: Arc::from(""),
        }
    }
}
