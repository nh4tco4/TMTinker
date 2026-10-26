use std::num::NonZeroU32;

use crate::core::{alphabet::ValidatedAlphabet, tape::BLANK};

use super::{
    document::SubmachineDocument,
    layout::LayoutPos,
    node::{Direction, NodeAction},
    submachine::{NodeKey, SubmachineKey},
    transition::TransitionSymbols,
};

fn all(alphabet: &ValidatedAlphabet) -> TransitionSymbols {
    TransitionSymbols::new(alphabet.iter().copied(), alphabet).expect("alphabet is nonempty")
}

fn symbol(character: char, alphabet: &ValidatedAlphabet) -> TransitionSymbols {
    TransitionSymbols::new([character], alphabet).expect("built-in symbol belongs to alphabet")
}

fn connect(
    machine: &mut SubmachineDocument,
    source: NodeKey,
    target: NodeKey,
    symbols: TransitionSymbols,
) {
    machine
        .add_edge(symbols, source, target)
        .expect("built-in nodes exist");
}

fn call(target: SubmachineKey, repeat: u32) -> NodeAction {
    NodeAction::Call {
        target,
        repeat: NonZeroU32::new(repeat).expect("built-in repeat is positive"),
    }
}

pub(super) fn move_word(
    name: &str,
    direction: Direction,
    alphabet: &ValidatedAlphabet,
) -> SubmachineDocument {
    let mut machine = SubmachineDocument::new_system(name);
    let start = machine.add_node(LayoutPos::new(450., 450.), NodeAction::Start);
    let movement = machine.add_node(
        LayoutPos::new(675., 450.),
        NodeAction::Move {
            direction,
            repeat: NonZeroU32::MIN,
        },
    );
    let stop = machine.add_node(LayoutPos::new(900., 450.), NodeAction::Stop);
    connect(&mut machine, start, movement, all(alphabet));
    connect(&mut machine, movement, stop, symbol(BLANK, alphabet));
    let loop_edge = machine
        .add_edge(symbol(BLANK, alphabet), movement, movement)
        .expect("built-in nodes exist");
    machine
        .machine_mut()
        .get_edge_mut(loop_edge)
        .expect("built-in edge exists")
        .set_inverse(true);
    machine
}

pub(super) fn copy_word(
    left: SubmachineKey,
    right: SubmachineKey,
    alphabet: &ValidatedAlphabet,
) -> SubmachineDocument {
    let mut machine = SubmachineDocument::new_system("Copy");
    let start = machine.add_node(LayoutPos::new(375., 450.), NodeAction::Start);
    let find_left = machine.add_node(LayoutPos::new(525., 450.), call(left, 1));
    let next = machine.add_node(
        LayoutPos::new(675., 450.),
        NodeAction::Move {
            direction: Direction::Right,
            repeat: NonZeroU32::MIN,
        },
    );
    let finish = machine.add_node(LayoutPos::new(825., 450.), call(right, 1));
    let stop = machine.add_node(LayoutPos::new(975., 450.), NodeAction::Stop);
    connect(&mut machine, start, find_left, all(alphabet));
    connect(&mut machine, find_left, next, all(alphabet));
    connect(&mut machine, next, finish, symbol(BLANK, alphabet));
    connect(&mut machine, finish, stop, all(alphabet));

    let mut y = 675.;
    for character in alphabet.iter().copied().filter(|&c| c != BLANK) {
        let erase = machine.add_node(LayoutPos::new(825., y), NodeAction::Write(BLANK));
        let find_copy = machine.add_node(LayoutPos::new(975., y), call(right, 2));
        let write_copy = machine.add_node(LayoutPos::new(1125., y), NodeAction::Write(character));
        let return_left = machine.add_node(LayoutPos::new(1275., y), call(left, 2));
        let restore = machine.add_node(LayoutPos::new(1425., y), NodeAction::Write(character));
        connect(&mut machine, next, erase, symbol(character, alphabet));
        for (source, target) in [
            (erase, find_copy),
            (find_copy, write_copy),
            (write_copy, return_left),
            (return_left, restore),
            (restore, next),
        ] {
            connect(&mut machine, source, target, all(alphabet));
        }
        y += 150.;
    }
    machine
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::tape::Tape,
        diagram::{
            compile::compile,
            model::document::DiagramDocument,
            runtime::{DiagramRunSession, RunStatus},
        },
    };

    fn runnable(alphabet: &ValidatedAlphabet, name: &str) -> DiagramDocument {
        let mut document = DiagramDocument::new_with_builtins(alphabet.clone());
        let target = document
            .submachines()
            .find(|(_, m)| m.machine().name() == name)
            .expect("requested built-in exists")
            .0;
        let main = document
            .submachine_mut(document.entry_submachine())
            .expect("main machine exists");
        let start = main.add_node(LayoutPos::new(300., 450.), NodeAction::Start);
        let invoke = main.add_node(LayoutPos::new(450., 450.), call(target, 1));
        let stop = main.add_node(LayoutPos::new(600., 450.), NodeAction::Stop);
        connect(main, start, invoke, all(alphabet));
        connect(main, invoke, stop, all(alphabet));
        document.validate().expect("built-in document is valid");
        document
    }

    fn run(document: &DiagramDocument, tape: Tape) -> Tape {
        let mut session =
            DiagramRunSession::new(compile(document).expect("built-in graph compiles"), tape);
        for _ in 0..100_000 {
            session.step();
            if matches!(session.status(), RunStatus::Halted | RunStatus::Faulted) {
                break;
            }
        }
        assert_eq!(session.status(), RunStatus::Halted, "{:?}", session.fault());
        session.tape().clone()
    }

    #[test]
    fn copy_handles_empty_and_all_short_words_for_each_alphabet() {
        for symbols in [" |", " 01", " 012ABC"] {
            let alphabet =
                ValidatedAlphabet::new(symbols.chars()).expect("valid test alphabet or tape");
            let document = runnable(&alphabet, "Copy");
            let characters: Vec<_> = alphabet.iter().copied().filter(|&c| c != BLANK).collect();
            let mut words = vec![String::new()];
            let mut level = vec![String::new()];
            for _ in 0..4 {
                level = level
                    .iter()
                    .flat_map(|prefix| characters.iter().map(move |c| format!("{prefix}{c}")))
                    .collect();
                words.extend(level.iter().cloned());
            }
            for word in words {
                let initial = Tape::from_text(&format!("{word}(λ)"), -7, &alphabet)
                    .expect("valid test alphabet or tape");
                let text = if word.is_empty() {
                    "λ(λ)".to_owned()
                } else {
                    format!("{word}λ{word}(λ)")
                };
                let expected =
                    Tape::from_text(&text, -7, &alphabet).expect("valid test alphabet or tape");
                assert_eq!(run(&document, initial), expected, "word: {word:?}");
            }
        }
    }

    #[test]
    fn movement_stops_on_first_blank_after_at_least_one_step() {
        let alphabet = ValidatedAlphabet::new(" 01".chars()).expect("valid test alphabet or tape");
        for (name, initial, expected) in [
            ("L", "λ10(λ)", "(λ)10λ"),
            ("R", "(λ)10λ", "λ10(λ)"),
            ("L", "λ1(0)λ", "(λ)10λ"),
            ("R", "λ(1)0λ", "λ10(λ)"),
            ("L", "λ(λ)", "(λ)λ"),
            ("R", "(λ)λ", "λ(λ)"),
        ] {
            let document = runnable(&alphabet, name);
            assert_eq!(
                run(
                    &document,
                    Tape::from_text(initial, -3, &alphabet).expect("valid test alphabet or tape")
                ),
                Tape::from_text(expected, -3, &alphabet).expect("valid test alphabet or tape")
            );
        }
    }

    #[test]
    fn copy_does_not_reject_contents_to_the_right() {
        let alphabet = ValidatedAlphabet::new(" 01".chars()).expect("valid test alphabet or tape");
        let document = runnable(&alphabet, "Copy");
        let initial =
            Tape::from_text("10(λ)11λ", 0, &alphabet).expect("valid test alphabet or tape");
        let expected =
            Tape::from_text("10λ1110(λ)", 0, &alphabet).expect("valid test alphabet or tape");
        assert_eq!(run(&document, initial), expected);
    }
}
