use crate::core::tm::SubmachineKey;

#[derive(PartialEq, Clone, Debug)]
pub enum EditorTool {
    Edit(EditTools),
    Place(BasicActionTools),
    Submachine {
        name: String,
        key: SubmachineKey,
        power: u32,
    }
}

impl Default for EditorTool {
    fn default() -> Self {
        Self::Edit(EditTools::default())
    }
}

#[derive(PartialEq, Clone, Default, Debug)]
pub enum EditTools {
    #[default]
    Move,
    Delete,
    Link,
    Comment
}

#[derive(PartialEq, Clone, Debug)]
pub enum BasicActionTools {
    Left,
    Right,
    Write,
    Start,
    Stop
}
