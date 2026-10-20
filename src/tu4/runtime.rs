use std::sync::Arc;

use super::compile::{Action, CompiledTu4};
use crate::{
    core::tape::Tape,
    shared::runtime::{RunSession, RunStatus},
};

pub struct Tu4RunSession {
    compiled: Arc<CompiledTu4>,
    initial_tape: Tape,
    tape: Tape,
    state: usize,
    status: RunStatus,
    steps: u64,
    fault: Option<String>,
}

impl Tu4RunSession {
    pub fn state_number(&self) -> u32 {
        self.compiled.state_number(self.state)
    }
    pub fn line(&self) -> Option<usize> {
        self.compiled
            .instruction(self.state, self.tape.read())
            .map(|command| command.line)
    }
    pub fn run(&mut self) {
        if self.status == RunStatus::Paused {
            self.status = RunStatus::Running;
        }
    }
    pub fn pause(&mut self) {
        if self.status == RunStatus::Running {
            self.status = RunStatus::Paused;
        }
    }
    pub fn stop(&mut self) {
        self.status = RunStatus::Halted;
    }
    pub fn reset(&mut self) {
        self.tape = self.initial_tape.clone();
        self.state = self.compiled.initial;
        self.status = RunStatus::Paused;
        self.steps = 0;
        self.fault = None;
    }
    pub fn replace_tape(&mut self, tape: Tape) {
        self.initial_tape = tape;
        self.reset();
    }
}

impl RunSession for Tu4RunSession {
    type Program = Arc<CompiledTu4>;

    fn new(compiled: Self::Program, tape: Tape) -> Self {
        Self {
            state: compiled.initial,
            compiled,
            initial_tape: tape.clone(),
            tape,
            status: RunStatus::Paused,
            steps: 0,
            fault: None,
        }
    }
    fn tape(&self) -> &Tape {
        &self.tape
    }
    fn status(&self) -> RunStatus {
        self.status
    }
    fn steps(&self) -> u64 {
        self.steps
    }
    fn fault(&self) -> Option<&str> {
        self.fault.as_deref()
    }
    fn run(&mut self) {
        self.run();
    }
    fn pause(&mut self) {
        self.pause();
    }
    fn stop(&mut self) {
        self.stop();
    }
    fn reset(&mut self) {
        self.reset();
    }
    fn program(&self) -> Self::Program {
        self.compiled.clone()
    }
    fn step(&mut self) {
        if matches!(self.status, RunStatus::Halted | RunStatus::Faulted) {
            return;
        }
        let Some(command) = self.compiled.instruction(self.state, self.tape.read()) else {
            self.fault = Some(format!(
                "No command for state {} and symbol {:?}",
                self.state_number(),
                self.tape.read()
            ));
            self.status = RunStatus::Faulted;
            return;
        };
        match command.action {
            Action::Left => self.tape.move_left(),
            Action::Right => self.tape.move_right(),
            Action::Halt => self.status = RunStatus::Halted,
            Action::Write(symbol) => self.tape.write(symbol),
        }
        if self.status != RunStatus::Halted {
            self.state = command.next;
        }
        self.steps = self.steps.saturating_add(1);
    }
}
