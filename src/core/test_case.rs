use super::tape::Tape;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TestCase {
    pub initial_tape: Tape,
    pub expected_tape: Tape,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TestResult {
    Passed { steps: u64 },
    Failed { steps: u64 },
    Faulted(String),
    StepLimitReached,
}

impl TestResult {
    pub fn label(&self) -> String {
        match self {
            Self::Passed { steps } => format!("Passed · {steps} steps"),
            Self::Failed { steps } => format!("Failed · {steps} steps"),
            Self::Faulted(message) => format!("Fault: {message}"),
            Self::StepLimitReached => "Step limit reached".into(),
        }
    }
}
