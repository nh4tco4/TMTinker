use std::{collections::HashMap, sync::Arc};

use crate::core::alphabet::ValidatedAlphabet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Left,
    Right,
    Halt,
    Write(char),
}

pub struct Instruction {
    pub read: char,
    pub action: Action,
    pub next: usize,
    pub line: usize,
}

pub struct State {
    pub number: u32,
    start: usize,
    end: usize,
}

pub struct CompiledTu4 {
    instructions: Vec<Instruction>,
    states: Vec<State>,
    pub initial: usize,
}

impl CompiledTu4 {
    pub fn state_number(&self, state: usize) -> u32 {
        self.states
            .get(state)
            .expect("runtime state belongs to compiled program")
            .number
    }

    pub fn instruction(&self, state: usize, read: char) -> Option<&Instruction> {
        let state = self.states.get(state)?;
        let commands = self.instructions.get(state.start..state.end)?;
        let index = commands
            .binary_search_by_key(&read, |command| command.read)
            .ok()?;
        commands.get(index)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Line {line}: {message}")]
pub struct CompileError {
    pub line: usize,
    pub message: String,
}

impl CompileError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

struct Parsed {
    state: u32,
    read: char,
    action: Action,
    next: u32,
    line: usize,
}

pub fn compile(
    source: &str,
    alphabet: &ValidatedAlphabet,
) -> Result<Arc<CompiledTu4>, CompileError> {
    let mut parsed = Vec::new();
    for (index, text) in source.lines().enumerate() {
        let line = index + 1;
        let text = text.split_once("//").map_or(text, |(code, _)| code);
        if text.trim().is_empty() {
            continue;
        }
        parsed.push(parse_command(text, line, alphabet)?);
    }
    let initial_number = parsed
        .first()
        .ok_or_else(|| CompileError::new(1, "Program is empty"))?
        .state;
    parsed.sort_unstable_by_key(|command| (command.state, command.read, command.line));
    let mut state_ids = HashMap::new();
    let mut states: Vec<State> = Vec::new();
    for (previous, command) in parsed.iter().zip(parsed.iter().skip(1)) {
        if previous.state == command.state && previous.read == command.read {
            return Err(CompileError::new(
                command.line,
                format!(
                    "Duplicate command for state {} and symbol {:?} (first on line {})",
                    command.state, command.read, previous.line,
                ),
            ));
        }
    }
    for (index, command) in parsed.iter().enumerate() {
        if states
            .last()
            .is_none_or(|state| state.number != command.state)
        {
            state_ids.insert(command.state, states.len());
            states.push(State {
                number: command.state,
                start: index,
                end: index,
            });
        }
        states.last_mut().expect("state was inserted").end = index + 1;
    }
    let initial = *state_ids
        .get(&initial_number)
        .expect("initial state was indexed");
    let mut instructions = Vec::with_capacity(parsed.len());
    for command in parsed {
        let next = if command.action == Action::Halt {
            initial
        } else {
            *state_ids.get(&command.next).ok_or_else(|| {
                CompileError::new(
                    command.line,
                    format!("Target state {} does not exist", command.next),
                )
            })?
        };
        instructions.push(Instruction {
            read: command.read,
            action: command.action,
            next,
            line: command.line,
        });
    }
    Ok(Arc::new(CompiledTu4 {
        instructions,
        states,
        initial,
    }))
}

fn parse_command(
    text: &str,
    line: usize,
    alphabet: &ValidatedAlphabet,
) -> Result<Parsed, CompileError> {
    let mut fields = text.split(',');
    let state = fields
        .next()
        .unwrap_or_default()
        .trim()
        .parse::<u32>()
        .map_err(|error| {
            CompileError::new(line, format!("Expected a u32 state number: {error}"))
        })?;
    let read = symbol(fields.next(), line, alphabet)?;
    let action_text = fields
        .next()
        .ok_or_else(|| CompileError::new(line, "Expected an action"))?;
    let action = match action_text.trim() {
        "<" => Action::Left,
        ">" => Action::Right,
        "~" => Action::Halt,
        _ => Action::Write(symbol(Some(action_text), line, alphabet)?),
    };
    let next_text = fields
        .next()
        .ok_or_else(|| CompileError::new(line, "Expected a fourth field"))?;
    let next = if action == Action::Halt {
        0 // Halt ignores the fourth field, including references to missing states.
    } else {
        next_text.trim().parse::<u32>().map_err(|error| {
            CompileError::new(line, format!("Expected a u32 target state: {error}"))
        })?
    };
    if fields.next().is_some() {
        return Err(CompileError::new(
            line,
            "Expected four comma-separated fields",
        ));
    }
    Ok(Parsed {
        state,
        read,
        action,
        next,
        line,
    })
}

fn symbol(
    field: Option<&str>,
    line: usize,
    alphabet: &ValidatedAlphabet,
) -> Result<char, CompileError> {
    let field = field.ok_or_else(|| CompileError::new(line, "Expected a symbol"))?;
    // Preserve a literal blank while allowing spacing around nonblank symbols.
    // I guess I will remove that later.
    let field = if !field.is_empty() && field.chars().all(|c| c == ' ') {
        " "
    } else {
        field.trim()
    };
    let mut chars = field.chars();
    let symbol = chars
        .next()
        .filter(|_| chars.next().is_none())
        .ok_or_else(|| {
            CompileError::new(line, "Expected one symbol (use λ or a space for blank)")
        })?;
    let symbol = if symbol == 'λ' { ' ' } else { symbol };
    if !alphabet.contains(symbol) {
        return Err(CompileError::new(
            line,
            format!("Symbol {symbol:?} is outside the project alphabet"),
        ));
    }
    Ok(symbol)
}
