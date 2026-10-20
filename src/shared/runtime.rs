use crate::core::tape::Tape;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunStatus {
    Paused,
    Running,
    Halted,
    Faulted,
}

pub trait RunSession {
    type Program: Clone;

    fn new(program: Self::Program, tape: Tape) -> Self;
    fn tape(&self) -> &Tape;
    fn status(&self) -> RunStatus;
    fn steps(&self) -> u64;
    fn fault(&self) -> Option<&str>;
    fn step(&mut self);
    fn run(&mut self);
    fn pause(&mut self);
    fn stop(&mut self);
    fn reset(&mut self);
    fn program(&self) -> Self::Program;
}
