use crate::diagram::model::submachine::SubmachineKey;

#[derive(PartialEq, Clone, Debug)]
pub enum EditorTool {
    Edit(EditTools),
    Place(BasicActionTools),
    Submachine(SubmachineKey),
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
    Comment,
}

#[derive(PartialEq, Clone, Debug)]
pub enum BasicActionTools {
    Left,
    Right,
    Write,
    Start,
    Stop,
}
