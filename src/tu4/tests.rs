use std::{fmt::Write as _, sync::Arc};

use super::{compile, convert, runtime::Tu4RunSession};
use crate::{
    core::{
        alphabet::ValidatedAlphabet,
        tape::Tape,
        test_case::{TestCase, TestResult},
    },
    diagram::{
        compile::compile as compile_diagram,
        model::{
            document::DiagramDocument,
            layout::LayoutPos,
            node::{Direction, NodeAction},
            submachine::{NodeKey, SubmachineKey, SubmachineName},
            transition::TransitionSymbols,
        },
        runtime::DiagramRunSession,
    },
    persistence,
    project::{
        Project,
        model::{ProjectEdit, ProjectName, TransactionKey},
    },
    shared::{
        checks::BatchCheck,
        runtime::{RunSession as _, RunStatus},
    },
};

fn alphabet() -> ValidatedAlphabet {
    ValidatedAlphabet::new(" 01#=|/".chars()).expect("valid alphabet")
}

fn session(source: &str) -> Tu4RunSession {
    Tu4RunSession::new(
        compile::compile(source, &alphabet()).expect("valid program"),
        Tape::new(),
    )
}

#[test]
fn sparse_u32_states_comments_blanks_and_source_lines() {
    let mut run = session("// comment\n4294967295,λ,>,7 // move\n\n7, ,1,0\n0,1,~,missing\n");
    assert_eq!(run.state_number(), u32::MAX);
    assert_eq!(run.line(), Some(2));
    run.step();
    assert_eq!(run.tape().head_position(), 1);
    assert_eq!(run.line(), Some(4));
    run.step();
    assert_eq!(run.tape().read(), '1');
    assert_eq!(run.line(), Some(5));
    run.step();
    assert_eq!(run.status(), RunStatus::Halted);
    assert_eq!(run.steps(), 3);
    run.step();
    assert_eq!(run.steps(), 3);
    run.reset();
    assert_eq!(run.status(), RunStatus::Paused);
    assert_eq!(run.tape(), &Tape::new());
    assert_eq!(run.state_number(), u32::MAX);
}

#[test]
fn same_symbol_same_state_loops_and_missing_commands_fault() {
    let mut run = session("0,λ,λ,0");
    for _ in 0..10 {
        run.step();
    }
    assert_eq!(run.steps(), 10);
    assert_eq!(run.status(), RunStatus::Paused);
    let mut run = session("0,λ,<,1\n1,1,~,0");
    run.step();
    assert_eq!(run.tape().head_position(), -1);
    assert_eq!(run.line(), None);
    run.step();
    assert_eq!(run.status(), RunStatus::Faulted);
    assert!(
        run.fault()
            .expect("missing-command error")
            .contains("state 1")
    );
    assert_eq!(run.steps(), 1);
}

#[test]
fn validates_syntax_duplicates_targets_and_alphabet_without_legacy_operators() {
    for text in [
        "",
        "// only a comment",
        "-1,λ,~,0",
        "4294967296,λ,~,0",
        "1,λ,>,2",
        "1,λ,1,1\n1, ,~,0",
        "1,λ,xx,1",
        "1,λ,#,1,2",
        "1,,~,0",
        "1,λ,~,0\n2,λ,0,-1",
        "1,λ,##,1",
    ] {
        assert!(
            compile::compile(text, &alphabet()).is_err(),
            "accepted {text:?}"
        );
    }
    for symbol in ['#', '=', '|', '/'] {
        let mut run = session(&format!("1,λ,{symbol},2\n2,{symbol},~,0"));
        run.step();
        assert_eq!(run.tape().read(), symbol);
        run.step();
        assert_eq!(run.status(), RunStatus::Halted);
    }
}

#[test]
fn four_hundred_thousand_commands_compile_and_execute() {
    let mut source = String::with_capacity(10_000_000);
    for state in 0..400_000u32 {
        if state == 399_999 {
            writeln!(source, "{state},λ,~,0").expect("String write");
        } else {
            writeln!(source, "{state},λ,λ,{}", state + 1).expect("String write");
        }
    }
    let mut run = session(&source);
    for _ in 0..400_000 {
        run.step();
    }
    assert_eq!(run.status(), RunStatus::Halted);
    assert_eq!(run.steps(), 400_000);
    assert_eq!(run.tape(), &Tape::new());
}

#[test]
fn shared_checks_report_pass_failure_fault_and_limit() {
    let compiled = compile::compile("0,λ,1,1\n1,1,~,0", &alphabet()).expect("valid program");
    let tests = [
        Some(TestCase {
            initial_tape: Tape::new(),
            expected_tape: Tape::from_text("(1)", -5, &alphabet()).expect("valid tape"),
        }),
        Some(TestCase {
            initial_tape: Tape::new(),
            expected_tape: Tape::new(),
        }),
        Some(TestCase {
            initial_tape: Tape::from_text("(0)", 0, &alphabet()).expect("valid tape"),
            expected_tape: Tape::new(),
        }),
        None,
        None,
        None,
    ];
    let mut checks = BatchCheck::<Tu4RunSession>::new(compiled, tests, 2);
    while !checks.is_done() {
        checks.tick();
    }
    assert_eq!(checks.results[0], Some(TestResult::Passed { steps: 2 }));
    assert_eq!(checks.results[1], Some(TestResult::Failed { steps: 2 }));
    assert!(matches!(checks.results[2], Some(TestResult::Faulted(_))));
    let tests = std::array::from_fn(|_| {
        Some(TestCase {
            initial_tape: Tape::new(),
            expected_tape: Tape::new(),
        })
    });
    let mut checks = BatchCheck::<Tu4RunSession>::new(
        compile::compile("0,λ,λ,0", &alphabet()).expect("loop compiles"),
        tests,
        17,
    );
    while !checks.is_done() {
        checks.tick();
    }
    assert!(
        checks
            .results
            .iter()
            .all(|result| *result == Some(TestResult::StepLimitReached))
    );
}

#[test]
fn source_history_and_mixed_library_preserve_invalid_drafts() {
    let mut project = Project::new_tu4(
        ProjectName::new("TU4".into()).expect("valid name"),
        alphabet(),
    );
    project
        .execute(
            ProjectEdit::SetSource(Arc::from("incomplete draft // keep me")),
            Some(TransactionKey::Source),
        )
        .expect("edit source");
    project.finish_transaction();
    let loaded =
        persistence::decode_project(&persistence::encode_project(&project).expect("encode TU4"))
            .expect("decode TU4");
    assert_eq!(
        loaded.tu4().expect("TU4").source.as_ref(),
        "incomplete draft // keep me"
    );
    assert!(project.undo());
    assert!(project.tu4().expect("TU4").source.is_empty());
    assert!(project.redo());
    let diagram = Project::new(
        ProjectName::new("Diagram".into()).expect("valid name"),
        DiagramDocument::new(alphabet()),
    );
    let library = persistence::decode_library(
        &persistence::encode_library([&diagram, &loaded].into_iter()).expect("encode library"),
    )
    .expect("decode mixed library");
    assert!(!library.first().expect("diagram entry").is_tu4());
    assert!(library.get(1).expect("TU4 entry").is_tu4());
    assert!(
        project
            .execute(
                ProjectEdit::AddMachine(SubmachineName::parse("Wrong".into()).expect("name")),
                None
            )
            .is_err()
    );
}

fn add(document: &mut DiagramDocument, key: SubmachineKey, action: NodeAction) -> NodeKey {
    document
        .submachine_mut(key)
        .expect("machine exists")
        .add_node(LayoutPos::new(0., 0.), action)
}
fn edge(
    document: &mut DiagramDocument,
    key: SubmachineKey,
    source: NodeKey,
    target: NodeKey,
    symbols: &[char],
) {
    let symbols = TransitionSymbols::new(symbols.iter().copied(), document.alphabet())
        .expect("valid symbols");
    document
        .submachine_mut(key)
        .expect("machine exists")
        .add_edge(symbols, source, target)
        .expect("valid edge");
}
fn all(document: &mut DiagramDocument, key: SubmachineKey, source: NodeKey, target: NodeKey) {
    let symbols: Vec<_> = document.alphabet().iter().copied().collect();
    edge(document, key, source, target, &symbols);
}
fn compare(document: &DiagramDocument, tape: Tape) {
    let mut diagram = DiagramRunSession::new(
        compile_diagram(document).expect("diagram compiles"),
        tape.clone(),
    );
    let converted = convert::convert(document).expect("diagram converts");
    let mut tu4 = Tu4RunSession::new(
        compile::compile(&converted.source, &converted.alphabet)
            .expect("converted source compiles"),
        tape,
    );
    for _ in 0..1000 {
        if diagram.status() == RunStatus::Halted {
            break;
        }
        diagram.step();
    }
    for _ in 0..10_000 {
        if tu4.status() == RunStatus::Halted {
            break;
        }
        tu4.step();
    }
    assert_eq!(diagram.status(), RunStatus::Halted);
    assert_eq!(tu4.status(), RunStatus::Halted, "{:?}", tu4.fault());
    assert_eq!(tu4.tape(), diagram.tape());
}

#[test]
fn conversion_preserves_write_branches_and_repeated_move_destinations() {
    let mut document = DiagramDocument::new(alphabet());
    let key = document.entry_submachine();
    let start = add(&mut document, key, NodeAction::Start);
    let write = add(&mut document, key, NodeAction::Write('1'));
    let right = add(
        &mut document,
        key,
        NodeAction::Move {
            direction: Direction::Right,
            repeat: 3.try_into().expect("positive"),
        },
    );
    let left = add(
        &mut document,
        key,
        NodeAction::Move {
            direction: Direction::Left,
            repeat: 2.try_into().expect("positive"),
        },
    );
    let stop = add(&mut document, key, NodeAction::Stop);
    all(&mut document, key, start, write);
    edge(&mut document, key, write, right, &['1']);
    edge(
        &mut document,
        key,
        write,
        stop,
        &[' ', '0', '#', '=', '|', '/'],
    );
    edge(&mut document, key, right, left, &['0']);
    edge(
        &mut document,
        key,
        right,
        stop,
        &[' ', '1', '#', '=', '|', '/'],
    );
    all(&mut document, key, left, stop);
    for text in ["(λ)110", "(0)111", "(1)000", "(λ)"] {
        compare(
            &document,
            Tape::from_text(text, -7, document.alphabet()).expect("valid tape"),
        );
    }
}

#[test]
fn conversion_and_parser_preserve_all_eighty_existing_symbols() {
    let symbols = std::iter::once(' ')
        .chain(('!'..='~').filter(|&symbol| ValidatedAlphabet::new([' ', symbol]).is_ok()));
    let alphabet = ValidatedAlphabet::new(symbols).expect("existing alphabet");
    assert_eq!(alphabet.len(), 80);
    for reserved in ['<', '>', '~'] {
        assert!(!alphabet.contains(reserved));
    }
    let mut document = DiagramDocument::new(alphabet.clone());
    let key = document.entry_submachine();
    let start = add(&mut document, key, NodeAction::Start);
    let stop = add(&mut document, key, NodeAction::Stop);
    all(&mut document, key, start, stop);
    for &symbol in alphabet.iter() {
        let mut tape = Tape::new();
        tape.set_cell(0, symbol);
        compare(&document, tape);
    }
}

#[test]
fn conversion_rejects_unrepresentable_repeat_counts_before_expansion() {
    let mut document = DiagramDocument::new(alphabet());
    let key = document.entry_submachine();
    let start = add(&mut document, key, NodeAction::Start);
    let movement = add(
        &mut document,
        key,
        NodeAction::Move {
            direction: Direction::Right,
            repeat: u32::MAX.try_into().expect("positive"),
        },
    );
    let stop = add(&mut document, key, NodeAction::Stop);
    all(&mut document, key, start, movement);
    all(&mut document, key, movement, stop);
    assert!(
        convert::convert(&document)
            .err()
            .expect("state overflow")
            .contains("u32")
    );
}

#[test]
fn conversion_preserves_nested_repeated_calls_and_pre_call_return_branch() {
    let mut document = DiagramDocument::new(alphabet());
    let main = document.entry_submachine();
    let child = document.add_submachine(SubmachineName::parse("Child".into()).expect("name"));
    let leaf = document.add_submachine(SubmachineName::parse("Leaf".into()).expect("name"));
    let s = add(&mut document, leaf, NodeAction::Start);
    let w = add(&mut document, leaf, NodeAction::Write('1'));
    let e = add(&mut document, leaf, NodeAction::Stop);
    all(&mut document, leaf, s, w);
    all(&mut document, leaf, w, e);
    let s = add(&mut document, child, NodeAction::Start);
    let c = add(
        &mut document,
        child,
        NodeAction::Call {
            target: leaf,
            repeat: 2.try_into().expect("positive"),
        },
    );
    let m = add(
        &mut document,
        child,
        NodeAction::Move {
            direction: Direction::Right,
            repeat: 1.try_into().expect("positive"),
        },
    );
    let e = add(&mut document, child, NodeAction::Stop);
    all(&mut document, child, s, c);
    all(&mut document, child, c, m);
    all(&mut document, child, m, e);
    let s = add(&mut document, main, NodeAction::Start);
    let c = add(
        &mut document,
        main,
        NodeAction::Call {
            target: child,
            repeat: 2.try_into().expect("positive"),
        },
    );
    let a = add(&mut document, main, NodeAction::Write('#'));
    let b = add(&mut document, main, NodeAction::Write('='));
    let e = add(&mut document, main, NodeAction::Stop);
    all(&mut document, main, s, c);
    edge(&mut document, main, c, a, &['0']);
    edge(&mut document, main, c, b, &[' ', '1', '#', '=', '|', '/']);
    all(&mut document, main, a, e);
    all(&mut document, main, b, e);
    for text in ["(0)11", "(1)00", "(λ)"] {
        compare(
            &document,
            Tape::from_text(text, -2, document.alphabet()).expect("tape"),
        );
    }
}

#[test]
fn recursion_is_rejected_by_edits_import_and_compiler() {
    let mut document = DiagramDocument::new(alphabet());
    let main = document.entry_submachine();
    let child = document.add_submachine(SubmachineName::parse("Child".into()).expect("name"));
    add(
        &mut document,
        main,
        NodeAction::Call {
            target: child,
            repeat: 1.try_into().expect("positive"),
        },
    );
    let mut project = Project::new(
        ProjectName::new("Cycle".into()).expect("name"),
        document.clone(),
    );
    let before = project.data().clone();
    let mut draft = document.submachine(child).expect("child").clone();
    draft.add_node(
        LayoutPos::new(0., 0.),
        NodeAction::Call {
            target: main,
            repeat: 1.try_into().expect("positive"),
        },
    );
    assert!(
        project
            .execute(
                ProjectEdit::ReplaceMachine {
                    key: child,
                    machine: Box::new(draft.clone())
                },
                None
            )
            .is_err()
    );
    assert!(project.data() == &before);
    document.replace_submachine(child, draft).expect("replace");
    assert!(document.validate_calls().is_err());
    assert!(compile_diagram(&document).is_err());
    assert!(convert::convert(&document).is_err());
    let invalid = Project::new(ProjectName::new("Cycle".into()).expect("name"), document);
    assert!(
        persistence::decode_project(&persistence::encode_project(&invalid).expect("encode"))
            .is_err()
    );
    let mut direct = DiagramDocument::new(alphabet());
    let key = direct.entry_submachine();
    add(
        &mut direct,
        key,
        NodeAction::Call {
            target: key,
            repeat: 1.try_into().expect("positive"),
        },
    );
    assert!(direct.validate_calls().is_err());
}
