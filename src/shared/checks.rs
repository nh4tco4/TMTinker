use crate::core::test_case::{TestCase, TestResult};
use crate::shared::runtime::{RunSession, RunStatus};
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

pub struct BatchCheck<S: RunSession> {
    compiled: S::Program,
    tests: [Option<TestCase>; 6],
    limit: u64,
    slot: usize,
    session: Option<S>,
    pub results: [Option<TestResult>; 6],
}
impl<S: RunSession> BatchCheck<S> {
    pub fn new(compiled: S::Program, tests: [Option<TestCase>; 6], limit: u64) -> Self {
        Self {
            compiled,
            tests,
            limit,
            slot: 0,
            session: None,
            results: std::array::from_fn(|_| None),
        }
    }
    pub fn is_done(&self) -> bool {
        self.slot == 6
    }
    pub fn current_slot(&self) -> usize {
        self.slot
    }
    pub fn tick(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        let start = Instant::now();
        for _ in 0..4096 {
            if self.is_done() {
                break;
            }
            #[cfg(not(target_arch = "wasm32"))]
            if start.elapsed() >= Duration::from_millis(4) {
                break;
            }
            let Some(test) = self
                .tests
                .get(self.slot)
                .expect("active test slot is in range")
            else {
                self.slot += 1;
                continue;
            };
            let session = self
                .session
                .get_or_insert_with(|| S::new(self.compiled.clone(), test.initial_tape.clone()));
            let result = if session.status() == RunStatus::Halted {
                Some(if session.tape().matches_output(&test.expected_tape) {
                    TestResult::Passed {
                        steps: session.steps(),
                    }
                } else {
                    TestResult::Failed {
                        steps: session.steps(),
                    }
                })
            } else if session.status() == RunStatus::Faulted {
                Some(TestResult::Faulted(
                    session.fault().unwrap_or("Execution failed").into(),
                ))
            } else if session.steps() >= self.limit {
                Some(TestResult::StepLimitReached)
            } else {
                session.step();
                None
            };
            if let Some(result) = result {
                *self
                    .results
                    .get_mut(self.slot)
                    .expect("active test slot is in range") = Some(result);
                self.slot += 1;
                self.session = None;
            }
        }
    }
}
