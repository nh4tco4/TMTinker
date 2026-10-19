#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunStatus {
    Paused,
    Running,
    Halted,
    Faulted,
}
