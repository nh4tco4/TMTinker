use super::*;
use crate::{
    core::{
        alphabet::ValidatedAlphabet,
        tape::Tape,
        test_case::{TestCase, TestResult},
    },
    diagram::model::{
        document::DiagramDocument, layout::LayoutPos, node::NodeAction, submachine::SubmachineName,
        transition::TransitionSymbols,
    },
    project::model::ProjectName,
};
use eframe::App as _;

fn fixture(looping: bool) -> Project {
    let alphabet = ValidatedAlphabet::new([' ', '0', '1']).unwrap();
    let mut document = DiagramDocument::new(alphabet.clone());
    let key = document.entry_submachine();
    let machine = document.submachine_mut(key).unwrap();
    let start = machine.add_node(LayoutPos::new(450., 225.), NodeAction::Start);
    let write = machine.add_node(LayoutPos::new(600., 225.), NodeAction::Write('1'));
    let stop = machine.add_node(LayoutPos::new(750., 225.), NodeAction::Stop);
    let symbols = || TransitionSymbols::new(alphabet.iter().copied(), &alphabet).unwrap();
    machine.add_edge(symbols(), start, write).unwrap();
    machine
        .add_edge(symbols(), write, if looping { write } else { stop })
        .unwrap();
    Project::new(ProjectName::new("Test project".into()).unwrap(), document)
}

#[test]
fn editor_start_to_finish_link_halts_without_changing_any_tape() {
    use crate::diagram::{
        editor::{
            basic_node_tools::{start_tool, stop_tool},
            edit_tools::link_tool,
            session::SelectableKey,
        },
        runtime::{DiagramRunSession, RunStatus},
    };

    let alphabet = ValidatedAlphabet::new([' ', '0', '1']).unwrap();
    let mut document = DiagramDocument::new(alphabet.clone());
    let machine = document
        .submachine_mut(document.entry_submachine())
        .unwrap();
    let start = start_tool(machine, egui::pos2(450., 225.));
    let stop = stop_tool(machine, egui::pos2(600., 225.));
    let mut source = None;
    link_tool(
        machine,
        &mut source,
        &alphabet,
        Some(SelectableKey::Node(start)),
    );
    link_tool(
        machine,
        &mut source,
        &alphabet,
        Some(SelectableKey::Node(stop)),
    );
    let project = Project::new(
        ProjectName::new("Start to finish".into()).unwrap(),
        document,
    );
    let project =
        persistence::decode_project(&persistence::encode_project(&project).unwrap()).unwrap();
    let compiled = compile(project.document()).expect("new editor link covers the whole alphabet");
    for text in ["(λ)", "1(0)1", "0(1)0", "1(λ)0"] {
        let tape = Tape::from_text(text, -2, &alphabet).unwrap();
        let mut session = DiagramRunSession::new(compiled.clone(), tape.clone());
        session.run();
        session.step();
        assert_eq!(session.source_location().node, stop);
        session.step();
        assert_eq!(session.status(), RunStatus::Halted);
        assert_eq!(session.tape(), &tape);
    }
}

#[test]
fn history_groups_a_drag_and_restores_slot_keys() {
    let mut project = fixture(false);
    let original = project.data().clone();
    let key = project.document().entry_submachine();
    let node = project
        .document()
        .submachine(key)
        .unwrap()
        .machine()
        .nodes_iter()
        .next()
        .unwrap()
        .0;
    for delta in [10., 20., 30.] {
        let mut draft = project.document().submachine(key).unwrap().clone();
        draft
            .layout_mut()
            .update_node_position_unsnapped(node, LayoutPos::new(delta, 0.));
        project
            .execute(
                ProjectEdit::ReplaceMachine {
                    key,
                    machine: Box::new(draft),
                },
                Some(TransactionKey::Canvas(key)),
            )
            .unwrap();
    }
    let moved = project.data().clone();
    project.finish_transaction();
    assert!(project.undo());
    assert!(project.data() == &original);
    assert!(!project.can_undo());
    assert!(project.redo());
    assert!(project.data() == &moved);
    project.undo();
    project
        .execute(
            ProjectEdit::AddMachine(SubmachineName::parse("Child".into()).unwrap()),
            None,
        )
        .unwrap();
    assert!(!project.can_redo());
}

#[test]
fn history_tracks_tests_and_prepared_tape() {
    let mut project = fixture(false);
    let tape = Tape::from_text("0(1)", -3, project.document().alphabet()).unwrap();
    let test = TestCase {
        initial_tape: tape.clone(),
        expected_tape: Tape::new(),
    };
    project
        .execute(
            ProjectEdit::SetTest {
                slot: 5,
                test: Some(test.clone()),
            },
            None,
        )
        .unwrap();
    project
        .execute(ProjectEdit::SetTape(tape.clone()), None)
        .unwrap();
    assert!(project.undo());
    assert_eq!(project.data().prepared_tape, Tape::new());
    assert_eq!(project.data().tests[5], Some(test));
    assert!(project.undo());
    assert!(project.data().tests[5].is_none());
    assert!(project.redo());
    assert!(project.redo());
    assert_eq!(project.data().prepared_tape, tape);
}

#[test]
fn project_and_library_roundtrip_preserve_data_without_history() {
    let mut project = fixture(false);
    let tape = Tape::from_text("1λ(0)", -8, project.document().alphabet()).unwrap();
    project
        .execute(ProjectEdit::SetTape(tape.clone()), None)
        .unwrap();
    project
        .execute(
            ProjectEdit::SetTest {
                slot: 0,
                test: Some(TestCase {
                    initial_tape: tape.clone(),
                    expected_tape: tape,
                }),
            },
            None,
        )
        .unwrap();
    let json = persistence::encode_project(&project).unwrap();
    let loaded = persistence::decode_project(&json).unwrap();
    assert!(loaded.data() == project.data());
    assert!(!loaded.can_undo());
    assert!(!loaded.can_redo());
    let json = persistence::encode_library([&project].into_iter()).unwrap();
    let loaded = persistence::decode_library(&json).unwrap();
    assert!(loaded[0].data() == project.data());
}

#[test]
fn invalid_imports_fail_without_mutating_existing_project() {
    let project = fixture(false);
    let original = project.data().clone();
    let mut value: serde_json::Value =
        serde_json::from_str(&persistence::encode_project(&project).unwrap()).unwrap();
    value["project"]["document"]["alphabet"]["characters"] = serde_json::json!([" ", "1", "1"]);
    assert!(persistence::decode_project(&value.to_string()).is_err());
    assert!(project.data() == &original);
    value["version"] = serde_json::json!(99);
    assert!(persistence::decode_project(&value.to_string()).is_err());
}

#[test]
fn legacy_files_import_and_validate_references() {
    let json = r#"{"name":"Old project","alphabet":[" ","1"],"submachines":[{"name":"Main","nodes":[{"action":"Start","position":{"x":300,"y":150}},{"action":"Stop","position":{"x":450,"y":150}}],"edges":[{"chars":[" ","1"],"source":0,"target":1}],"notes":[]}] }"#;
    let project = persistence::decode_project(json).unwrap();
    assert!(compile(project.document()).is_ok());
    assert!(persistence::decode_project(&json.replace("\"target\":1", "\"target\":9")).is_err());
}

#[test]
fn system_machine_commands_cannot_rename_replace_or_unlock() {
    let project = fixture(false);
    let mut value = serde_json::to_value(project.data()).unwrap();
    let slots = value["document"]["submachines"].as_array_mut().unwrap();
    for slot in slots {
        if let Some(machine) = slot.get_mut("value").filter(|value| value.is_object()) {
            machine["system"] = serde_json::json!(true);
        }
    }
    let mut project = Project::from_data(serde_json::from_value(value).unwrap());
    let key = project.document().entry_submachine();
    assert!(project.document().submachine(key).unwrap().is_system());
    assert!(
        project
            .execute(
                ProjectEdit::RenameMachine {
                    key,
                    name: SubmachineName::parse("Changed".into()).unwrap()
                },
                None
            )
            .is_err()
    );
    let draft = project.document().submachine(key).unwrap().clone();
    assert!(
        project
            .execute(
                ProjectEdit::ReplaceMachine {
                    key,
                    machine: Box::new(draft)
                },
                None
            )
            .is_err()
    );
    assert!(
        project
            .execute(ProjectEdit::RemoveMachine(key), None)
            .is_err()
    );
    assert!(!project.can_undo());
}

#[test]
fn deleting_active_submachine_returns_to_main_and_undo_restores_its_keys() {
    use crate::diagram::editor::tool::EditorTool;
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let mut document = fixture(false).document().clone();
    let main = document.entry_submachine();
    let child = document.add_submachine(SubmachineName::parse("Child".into()).unwrap());
    let node = document
        .submachine_mut(child)
        .unwrap()
        .add_node(LayoutPos::new(450., 225.), NodeAction::Start);
    let key = app.projects.insert(Project::new(
        ProjectName::new("Delete test".into()).unwrap(),
        document,
    ));
    let original = app.projects[key].data().clone();
    app.open(key).unwrap();
    app.dispatch(Command::OpenMachine(child), &ctx).unwrap();
    app.dispatch(Command::SelectTool(EditorTool::Submachine(child)), &ctx)
        .unwrap();
    if let Screen::Diagram(w) = &mut app.screen {
        w.ui.settings = Some(child);
    }
    frame(&mut app, &ctx, 1280., 820., 0., vec![]);
    let out = frame(&mut app, &ctx, 1280., 820., 0.1, vec![]);
    let pos = crate::ui::test_renderer::text_rects(&out)
        .iter()
        .find(|(text, _)| text == "Delete submachine")
        .unwrap()
        .1
        .center();
    for (time, pressed) in [(0.2, true), (0.3, false)] {
        frame(
            &mut app,
            &ctx,
            1280.,
            820.,
            time,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
    }
    assert!(app.projects[key].document().submachine(child).is_none());
    Project::validate(app.projects[key].data()).unwrap();
    let Screen::Diagram(w) = &app.screen else {
        panic!("editor workspace")
    };
    assert_eq!(w.editor.active_submachine(), main);
    assert_eq!(*w.editor.tool(), EditorTool::default());
    assert!(w.ui.settings.is_none());
    app.dispatch(Command::Invoke(CommandId::Undo), &ctx)
        .unwrap();
    assert!(app.projects[key].data() == &original);
    assert!(
        app.projects[key]
            .document()
            .submachine(child)
            .unwrap()
            .machine()
            .get_node(node)
            .is_some()
    );
    app.dispatch(Command::SelectTool(EditorTool::Submachine(child)), &ctx)
        .unwrap();
    app.dispatch(Command::Invoke(CommandId::Redo), &ctx)
        .unwrap();
    assert!(app.projects[key].document().submachine(child).is_none());
    let Screen::Diagram(w) = &app.screen else {
        panic!("editor workspace")
    };
    assert_eq!(*w.editor.tool(), EditorTool::default());
    frame(&mut app, &ctx, 1280., 820., 0.4, vec![]);
}

#[test]
fn deleting_main_or_called_submachine_keeps_the_project_unchanged() {
    let mut document = fixture(false).document().clone();
    let main = document.entry_submachine();
    let child = document.add_submachine(SubmachineName::parse("Child".into()).unwrap());
    document.submachine_mut(main).unwrap().add_node(
        LayoutPos::new(450., 375.),
        NodeAction::Call {
            target: child,
            repeat: std::num::NonZeroU32::MIN,
        },
    );
    let mut project = Project::new(ProjectName::new("Called child".into()).unwrap(), document);
    let original = project.data().clone();
    for target in [main, child] {
        assert!(
            project
                .execute(ProjectEdit::RemoveMachine(target), None)
                .is_err()
        );
        assert!(project.data() == &original);
        assert!(!project.can_undo());
        Project::validate(project.data()).unwrap();
    }
}

#[test]
fn batch_checks_both_symbols_and_head_and_keeps_runs_independent() {
    let project = fixture(false);
    let alphabet = project.document().alphabet();
    let initial = Tape::from_text("(0)", -2, alphabet).unwrap();
    let expected = Tape::from_text("(1)", -2, alphabet).unwrap();
    let mut tests = std::array::from_fn(|_| None);
    tests[0] = Some(TestCase {
        initial_tape: initial.clone(),
        expected_tape: expected.clone(),
    });
    let mut wrong_head = expected.clone();
    wrong_head.set_head(0);
    tests[1] = Some(TestCase {
        initial_tape: initial.clone(),
        expected_tape: wrong_head,
    });
    tests[2] = Some(TestCase {
        initial_tape: initial.clone(),
        expected_tape: initial,
    });
    tests[5] = Some(TestCase {
        initial_tape: Tape::new(),
        expected_tape: Tape::from_text("(1)", 0, alphabet).unwrap(),
    });
    let mut batch = BatchCheck::new(compile(project.document()).unwrap(), tests, 100);
    for _ in 0..100 {
        batch.tick();
        if batch.is_done() {
            break;
        }
    }
    assert!(batch.is_done());
    assert!(matches!(batch.results[0], Some(TestResult::Passed { .. })));
    assert!(matches!(batch.results[1], Some(TestResult::Failed { .. })));
    assert!(matches!(batch.results[2], Some(TestResult::Failed { .. })));
    assert!(batch.results[3].is_none());
    assert!(matches!(batch.results[5], Some(TestResult::Passed { .. })));
}

#[test]
fn batch_stops_a_loop_at_the_exact_step_limit() {
    let project = fixture(true);
    let mut tests = std::array::from_fn(|_| None);
    tests[0] = Some(TestCase {
        initial_tape: Tape::new(),
        expected_tape: Tape::new(),
    });
    let mut batch = BatchCheck::new(compile(project.document()).unwrap(), tests, 17);
    for _ in 0..100 {
        batch.tick();
        if batch.is_done() {
            break;
        }
    }
    assert!(batch.is_done());
    assert_eq!(batch.results[0], Some(TestResult::StepLimitReached));
}

#[test]
fn editing_tape_restarts_execution_and_updates_reset_baseline() {
    let project = fixture(false);
    let mut session = DiagramRunSession::new(compile(project.document()).unwrap(), Tape::new());
    session.run();
    session.step();
    session.step();
    assert_eq!(session.steps(), 2);
    let tape = Tape::from_text("(0)", -5, project.document().alphabet()).unwrap();
    session.replace_tape(tape.clone());
    assert_eq!(session.status(), RunStatus::Paused);
    assert_eq!(session.steps(), 0);
    assert_eq!(session.tape(), &tape);
    session.step();
    session.step();
    session.reset();
    assert_eq!(session.tape(), &tape);
}

#[test]
fn tape_text_roundtrip_keeps_negative_indices_and_blank_head() {
    let alphabet = ValidatedAlphabet::new([' ', '0', '1']).unwrap();
    let tape = Tape::from_text("1λ(λ)0", -7, &alphabet).unwrap();
    let (text, origin) = tape.to_text();
    assert_eq!(Tape::from_text(&text, origin, &alphabet).unwrap(), tape);
    assert!(Tape::from_text("01", 0, &alphabet).is_err());
    assert!(Tape::from_text("(0)(1)", 0, &alphabet).is_err());
}

#[test]
fn command_metadata_has_unique_ids_and_shortcuts() {
    let mut ids = std::collections::HashSet::new();
    for info in REGISTRY {
        assert!(ids.insert(info.id));
        assert!(!info.label.is_empty());
    }
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let key = app.projects.insert(fixture(false));
    for mode in 0..4 {
        match mode {
            1 => app.open(key).unwrap(),
            2 => app
                .dispatch(Command::Invoke(CommandId::OpenRunner), &ctx)
                .unwrap(),
            3 => app.dispatch(Command::Invoke(CommandId::Run), &ctx).unwrap(),
            _ => {}
        }
        let mut keys = std::collections::HashSet::new();
        for info in REGISTRY.iter().filter(|info| app.enabled(info.id)) {
            if let Some(key) = info.key {
                assert!(
                    keys.insert((key, info.modified)),
                    "duplicate shortcut in mode {mode}: {:?}",
                    info.id
                );
            }
        }
    }
}

#[derive(Default)]
struct MemoryStorage(std::collections::HashMap<String, String>);
impl eframe::Storage for MemoryStorage {
    fn get_string(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }
    fn set_string(&mut self, key: &str, value: String) {
        self.0.insert(key.into(), value);
    }
    fn flush(&mut self) {}
}
#[test]
fn saving_recovers_projects_and_protects_an_unreadable_previous_library() {
    let ctx = egui::Context::default();
    let mut storage = MemoryStorage::default();
    storage
        .0
        .insert(persistence::STORAGE_KEY.into(), "broken file".into());
    let mut cc = eframe::CreationContext::_new_kittest(ctx.clone());
    cc.storage = Some(&storage);
    let mut app = App::new(&cc);
    assert!(app.projects.is_empty());
    assert!(app.notice.as_ref().unwrap().1);
    app.projects.insert(fixture(false));
    app.persist(&mut storage).unwrap();
    assert_eq!(storage.0.get("tmtinker.recovery").unwrap(), "broken file");
    let mut cc = eframe::CreationContext::_new_kittest(ctx);
    cc.storage = Some(&storage);
    let restored = App::new(&cc);
    assert_eq!(restored.projects.len(), 1);
    assert!(!restored.projects.values().next().unwrap().can_undo());
}

fn frame(
    app: &mut App,
    ctx: &egui::Context,
    width: f32,
    height: f32,
    time: f64,
    events: Vec<egui::Event>,
) -> egui::FullOutput {
    ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, height),
            )),
            time: Some(time),
            events,
            ..Default::default()
        },
        |ctx| app.update(ctx, &mut eframe::Frame::_new_kittest()),
    )
}

#[test]
fn repaints_do_not_accelerate_the_scheduled_runner() {
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let key = app.projects.insert(fixture(false));
    app.open(key).unwrap();
    app.dispatch(Command::Invoke(CommandId::OpenRunner), &ctx)
        .unwrap();
    app.dispatch(Command::Invoke(CommandId::Run), &ctx).unwrap();
    for time in [0.05, 0.1, 0.15, 0.2, 0.25] {
        frame(&mut app, &ctx, 1280., 820., time, vec![]);
    }
    let Screen::Diagram(w) = &app.screen else {
        panic!("runner workspace");
    };
    assert_eq!(w.runner.as_ref().unwrap().session.steps(), 0);
    frame(&mut app, &ctx, 1280., 820., 0.36, vec![]);
    let Screen::Diagram(w) = &app.screen else {
        panic!("runner workspace");
    };
    assert_eq!(w.runner.as_ref().unwrap().session.steps(), 1);
}

#[test]
fn pointer_drag_commits_one_project_history_entry() {
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let project = fixture(false);
    let original = project.data().clone();
    let key = app.projects.insert(project);
    app.open(key).unwrap();
    frame(&mut app, &ctx, 1280., 820., 0., vec![]);
    let pos = egui::pos2(450., 225.);
    frame(
        &mut app,
        &ctx,
        1280.,
        820.,
        0.1,
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    for (i, x) in [465., 480., 510.].into_iter().enumerate() {
        frame(
            &mut app,
            &ctx,
            1280.,
            820.,
            0.2 + i as f64 * 0.1,
            vec![egui::Event::PointerMoved(egui::pos2(x, 225.))],
        );
    }
    let pos = egui::pos2(510., 225.);
    frame(
        &mut app,
        &ctx,
        1280.,
        820.,
        0.6,
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert!(app.projects[key].data() != &original);
    app.dispatch(Command::Invoke(CommandId::Undo), &ctx)
        .unwrap();
    assert!(app.projects[key].data() == &original);
    assert!(!app.projects[key].can_undo());
}

#[test]
fn failed_storage_write_keeps_unsaved_data() {
    struct Unavailable;
    impl eframe::Storage for Unavailable {
        fn get_string(&self, _: &str) -> Option<String> {
            None
        }
        fn set_string(&mut self, _: &str, _: String) {}
        fn flush(&mut self) {}
    }
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx));
    app.projects.insert(fixture(false));
    app.dirty = true;
    assert!(app.persist(&mut Unavailable).is_err());
    assert!(app.dirty);
    assert_eq!(app.projects.len(), 1);
}

#[test]
fn imported_nested_calls_keep_targets_and_repeat_counts() {
    let mut data = fixture(false).data().clone();
    let alphabet = data.document.alphabet().clone();
    let parent = data
        .document
        .diagram_mut()
        .expect("diagram document")
        .entry_submachine();
    let child = data
        .document
        .diagram_mut()
        .expect("diagram document")
        .add_submachine(SubmachineName::parse("Child".into()).unwrap());
    let machine = data
        .document
        .diagram_mut()
        .expect("diagram document")
        .submachine_mut(child)
        .unwrap();
    let start = machine.add_node(LayoutPos::new(450., 375.), NodeAction::Start);
    let write = machine.add_node(LayoutPos::new(600., 375.), NodeAction::Write('0'));
    let stop = machine.add_node(LayoutPos::new(750., 375.), NodeAction::Stop);
    for (a, b) in [(start, write), (write, stop)] {
        machine
            .add_edge(
                TransitionSymbols::new(alphabet.iter().copied(), &alphabet).unwrap(),
                a,
                b,
            )
            .unwrap();
    }
    let machine = data
        .document
        .diagram_mut()
        .expect("diagram document")
        .submachine_mut(parent)
        .unwrap();
    let node = machine
        .machine()
        .nodes_iter()
        .find(|(_, node)| matches!(node.action(), NodeAction::Write(_)))
        .unwrap()
        .0;
    machine
        .machine_mut()
        .get_mut_node(node)
        .unwrap()
        .set_action(NodeAction::Call {
            target: child,
            repeat: std::num::NonZeroU32::new(2).unwrap(),
        });
    let project = Project::from_data(data);
    let loaded =
        persistence::decode_project(&persistence::encode_project(&project).unwrap()).unwrap();
    let mut session = DiagramRunSession::new(compile(loaded.document()).unwrap(), Tape::new());
    for _ in 0..30 {
        if session.status() == RunStatus::Halted {
            break;
        }
        session.step();
    }
    assert_eq!(session.status(), RunStatus::Halted);
    assert_eq!(session.tape().cell(0), '0');
}

#[test]
fn headless_ui_pages_and_pointer_commands() {
    use crate::ui::test_renderer::{TestRenderer, text_rects};
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let mut renderer = TestRenderer::default();
    let out = frame(&mut app, &ctx, 1280., 820., 0., vec![]);
    renderer.update(&out);
    let out = frame(&mut app, &ctx, 1280., 820., 0.1, vec![]);
    renderer.update(&out);
    let labels = text_rects(&out);
    let new_rect = labels
        .iter()
        .find(|(text, _)| text.contains("New project"))
        .unwrap()
        .1;
    let snapshots = std::env::var_os("TMTINKER_UI_SNAPSHOTS").map(std::path::PathBuf::from);
    if let Some(dir) = &snapshots {
        std::fs::create_dir_all(dir).unwrap();
        renderer.save(&ctx, out, 1280, 820, &dir.join("menu.ppm"));
    }
    let pos = new_rect.center();
    let out = frame(
        &mut app,
        &ctx,
        1280.,
        820.,
        0.2,
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    renderer.update(&out);
    let out = frame(
        &mut app,
        &ctx,
        1280.,
        820.,
        0.3,
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    renderer.update(&out);
    assert!(app.menu.new_project_open);
    let out = frame(&mut app, &ctx, 1280., 820., 0.35, vec![]);
    renderer.update(&out);
    let out = frame(&mut app, &ctx, 1280., 820., 0.4, vec![]);
    renderer.update(&out);
    assert!(
        text_rects(&out)
            .iter()
            .any(|(text, _)| text.contains("Quadruples"))
    );
    if let Some(dir) = &snapshots {
        renderer.save(&ctx, out, 1280, 820, &dir.join("create.ppm"));
    }
    app.menu.new_project_open = false;
    let key = app.projects.insert(fixture(false));
    app.open(key).unwrap();
    let out = frame(&mut app, &ctx, 1280., 820., 0.5, vec![]);
    renderer.update(&out);
    let out = frame(&mut app, &ctx, 1280., 820., 0.6, vec![]);
    renderer.update(&out);
    let labels = text_rects(&out);
    for label in ["Tools", "Submachines", "Validate"] {
        assert!(
            labels.iter().any(|(text, _)| text.contains(label)),
            "missing {label}"
        );
    }
    if let Some(dir) = &snapshots {
        renderer.save(&ctx, out, 1280, 820, &dir.join("editor.ppm"));
    }
    app.dispatch(Command::Invoke(CommandId::OpenRunner), &ctx)
        .unwrap();
    let out = frame(&mut app, &ctx, 1280., 820., 0.7, vec![]);
    renderer.update(&out);
    let out = frame(&mut app, &ctx, 1280., 820., 0.8, vec![]);
    renderer.update(&out);
    let labels = text_rects(&out);
    assert!(labels.iter().any(|(text, _)| text.contains("Tests")));
    let rect = |label: &str| labels.iter().find(|(text, _)| text == label).unwrap().1;
    assert!(rect("Tests").top() < rect("Tape").bottom());
    assert!(rect("Tape").left() > rect("Tests").right());
    assert!(rect("Tape").right() < rect("Run diagram").left());
    assert!(rect("Tests").left() < rect("Run diagram").left());
    assert!(rect("Editor").left() < rect("Save").left());
    assert!(
        (rect("Editor").top() - rect("Save").top()).abs() < 2.,
        "{:?} {:?}",
        rect("Editor"),
        rect("Save")
    );
    assert!(
        !labels
            .iter()
            .any(|(text, _)| text == "Undo" || text == "Redo")
    );
    assert!(rect(crate::ui::theme::NODE_START).top() > rect("Tape").bottom());
    assert_eq!(
        labels
            .iter()
            .filter(|(text, _)| text.contains("Set up test"))
            .count(),
        6
    );
    if let Some(dir) = &snapshots {
        renderer.save(&ctx, out, 1280, 820, &dir.join("runner.ppm"));
    }
    app.dispatch(Command::ScrollTape(-5), &ctx).unwrap();
    if let Screen::Diagram(w) = &app.screen {
        let runner = w.runner.as_ref().unwrap();
        assert_eq!(runner.ui.tape_center, -5);
        assert_eq!(runner.session.tape().head_position(), 0);
    }
    app.dispatch(Command::Invoke(CommandId::TapeHead), &ctx)
        .unwrap();
    let out = frame(&mut app, &ctx, 800., 560., 0.9, vec![]);
    renderer.update(&out);
    let out = frame(&mut app, &ctx, 800., 560., 1., vec![]);
    renderer.update(&out);
    let out = frame(&mut app, &ctx, 800., 560., 1.1, vec![]);
    renderer.update(&out);
    if let Some(dir) = &snapshots {
        renderer.save(&ctx, out, 800, 560, &dir.join("runner-small.ppm"));
    }
}

#[test]
fn checks_match_translated_output_and_reject_wrong_head_or_cells() {
    use crate::diagram::model::node::Direction;
    let alphabet = ValidatedAlphabet::new([' ', '0', '1']).unwrap();
    let mut document = DiagramDocument::new(alphabet.clone());
    let machine = document
        .submachine_mut(document.entry_submachine())
        .unwrap();
    let nodes = [
        NodeAction::Start,
        NodeAction::Move {
            direction: Direction::Left,
            repeat: 1.try_into().unwrap(),
        },
        NodeAction::Write('1'),
        NodeAction::Move {
            direction: Direction::Right,
            repeat: 1.try_into().unwrap(),
        },
        NodeAction::Stop,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, action)| {
        machine.add_node(LayoutPos::new(300. + index as f32 * 150., 225.), action)
    })
    .collect::<Vec<_>>();
    for pair in nodes.windows(2) {
        machine
            .add_edge(
                TransitionSymbols::new(alphabet.iter().copied(), &alphabet).unwrap(),
                pair[0],
                pair[1],
            )
            .unwrap();
    }
    let compiled = compile(&document).unwrap();
    let tests = ["1(λ)", "(1)λ", "0(λ)", "1(λ)1", "λ1(λ)λλ", "(λ)"].map(|expected| {
        Some(TestCase {
            initial_tape: Tape::new(),
            expected_tape: Tape::from_tape(expected, &alphabet).unwrap(),
        })
    });
    let mut batch = BatchCheck::new(compiled, tests, 100);
    while !batch.is_done() {
        batch.tick();
    }
    for (slot, passed) in [true, false, false, false, true, false]
        .into_iter()
        .enumerate()
    {
        assert_eq!(
            matches!(batch.results[slot], Some(TestResult::Passed { .. })),
            passed,
            "test {slot}"
        );
    }
}

fn pointer_click(app: &mut App, ctx: &egui::Context, pos: egui::Pos2, time: f64) {
    for (offset, pressed) in [(0., true), (0.01, false)] {
        frame(
            app,
            ctx,
            1280.,
            820.,
            time + offset,
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
    }
}

fn save_tu4_snapshot(ctx: &egui::Context, out: egui::FullOutput) {
    let mut renderer = crate::ui::test_renderer::TestRenderer::default();
    renderer.update(&out);
    if let Some(dir) = std::env::var_os("TMTINKER_UI_SNAPSHOTS") {
        let dir = std::path::PathBuf::from(dir);
        std::fs::create_dir_all(&dir).expect("snapshot directory");
        renderer.save(ctx, out, 1280, 820, &dir.join("tu4-runner.ppm"));
    }
}

#[test]
fn tu4_editor_and_runner_use_the_project_lifecycle() {
    use crate::ui::test_renderer::text_rects;
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    app.dispatch(
        Command::CreateTu4 {
            name: ProjectName::new("TU4 test".into()).expect("name"),
            alphabet: ValidatedAlphabet::new([' ', '0', '1']).expect("alphabet"),
        },
        &ctx,
    )
    .expect("create TU4");
    let out = frame(&mut app, &ctx, 1280., 820., 0.1, vec![]);
    let hint = text_rects(&out)
        .into_iter()
        .find(|(text, _)| text.contains("1,λ,>,2"))
        .expect("source editor hint")
        .1;
    pointer_click(&mut app, &ctx, hint.left_top() + egui::vec2(5., 5.), 0.2);
    frame(
        &mut app,
        &ctx,
        1280.,
        820.,
        0.3,
        vec![egui::Event::Paste("1,λ,1,2\n2,1,~,0".into())],
    );
    assert_eq!(
        app.project()
            .expect("project")
            .tu4()
            .expect("TU4")
            .source
            .as_ref(),
        "1,λ,1,2\n2,1,~,0"
    );
    app.dispatch(Command::Invoke(CommandId::OpenRunner), &ctx)
        .expect("open runner");
    assert!(!app.enabled(CommandId::EditSource));
    assert!(!app.enabled(CommandId::Undo));
    app.dispatch(
        Command::Edit {
            edit: ProjectEdit::SetSource(std::sync::Arc::from("blocked")),
            group: None,
        },
        &ctx,
    )
    .expect("blocked edit is ignored");
    let document = app.project().expect("project").tu4().expect("TU4");
    assert!(document.source.starts_with("1,λ"));
    let out = frame(&mut app, &ctx, 1280., 820., 0.4, vec![]);
    for label in ["Run TU4", "Tests", "Tape", "Line: 1", "Run without delay"] {
        assert!(
            text_rects(&out).iter().any(|(text, _)| text == label),
            "missing {label}"
        );
    }
    save_tu4_snapshot(&ctx, out);
    app.dispatch(Command::Invoke(CommandId::Step), &ctx)
        .expect("write");
    app.dispatch(Command::Invoke(CommandId::Step), &ctx)
        .expect("halt");
    let Screen::Tu4(w) = &app.screen else {
        panic!("TU4 screen");
    };
    assert_eq!(
        w.runner.as_ref().expect("runner").session.status(),
        RunStatus::Halted
    );
    app.dispatch(Command::Invoke(CommandId::BackToEditor), &ctx)
        .expect("return to editor");
    assert!(app.enabled(CommandId::EditSource));
    app.dispatch(Command::Invoke(CommandId::Undo), &ctx)
        .expect("undo source");
    assert!(
        app.project()
            .expect("project")
            .tu4()
            .expect("TU4")
            .source
            .is_empty()
    );
    app.dispatch(Command::Invoke(CommandId::Redo), &ctx)
        .expect("redo source");
    let mut storage = MemoryStorage::default();
    app.persist(&mut storage).expect("save TU4");
    let mut cc = eframe::CreationContext::_new_kittest(ctx.clone());
    cc.storage = Some(&storage);
    let restored = App::new(&cc);
    assert!(
        restored
            .projects
            .values()
            .next()
            .expect("restored project")
            .is_tu4()
    );
}

#[test]
fn conversion_button_opens_a_separate_tu4_project_with_tape_and_tests() {
    use crate::ui::test_renderer::text_rects;
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let mut diagram = fixture(false);
    let tape = Tape::from_text("(0)", -4, diagram.alphabet()).expect("tape");
    diagram
        .execute(ProjectEdit::SetTape(tape.clone()), None)
        .expect("set tape");
    diagram
        .execute(
            ProjectEdit::SetTest {
                slot: 0,
                test: Some(TestCase {
                    initial_tape: tape.clone(),
                    expected_tape: tape.clone(),
                }),
            },
            None,
        )
        .expect("set test");
    let key = app.projects.insert(diagram);
    app.open(key).expect("open diagram");
    let before = app
        .projects
        .get(key)
        .expect("original diagram")
        .data()
        .clone();
    let out = frame(&mut app, &ctx, 1280., 820., 0.8, vec![]);
    let button = text_rects(&out)
        .into_iter()
        .find(|(text, _)| text == "Open in TU4")
        .expect("conversion button")
        .1;
    pointer_click(&mut app, &ctx, button.center() - egui::vec2(0., 40.), 0.9);
    assert!(matches!(app.screen, Screen::Tu4(_)));
    let converted = app.project().expect("converted project");
    assert_eq!(converted.data().prepared_tape, tape);
    assert_eq!(converted.data().tests, before.tests);
    assert!(app.projects.get(key).expect("original diagram").data() == &before);
    assert!(
        crate::tu4::compile::compile(&converted.tu4().expect("TU4").source, converted.alphabet())
            .is_ok()
    );
}

#[test]
fn large_tu4_program_uses_paged_editing_and_virtualized_execution() {
    use crate::ui::test_renderer::text_rects;
    use std::fmt::Write as _;
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let mut project = Project::new_tu4(
        ProjectName::new("Large TU4".into()).expect("name"),
        ValidatedAlphabet::new([' ', '1']).expect("alphabet"),
    );
    let mut source = String::new();
    for state in 0..400_000 {
        writeln!(source, "{state},λ,λ,{}", state + 1).expect("String write");
    }
    source.push_str("400000,λ,~,0");
    project
        .execute(ProjectEdit::SetSource(std::sync::Arc::from(source)), None)
        .expect("source");
    let key = app.projects.insert(project);
    app.open(key).expect("open large project");
    let out = frame(&mut app, &ctx, 1280., 820., 0.1, vec![]);
    let labels = text_rects(&out);
    assert!(
        labels
            .iter()
            .any(|(text, _)| text.contains("Lines 1–1000 of 400001"))
    );
    app.dispatch(Command::Invoke(CommandId::OpenRunner), &ctx)
        .expect("compile large project");
    let out = frame(&mut app, &ctx, 1280., 820., 0.2, vec![]);
    assert!(
        text_rects(&out).len() < 250,
        "render only visible source lines"
    );
    assert!(
        text_rects(&out)
            .iter()
            .any(|(text, _)| text.contains("2500 lines"))
    );
    if let Screen::Tu4(w) = &mut app.screen {
        w.ui.shared.fast = true;
    }
    app.dispatch(Command::Invoke(CommandId::Run), &ctx)
        .expect("run without delay");
    frame(&mut app, &ctx, 1280., 820., 0.3, vec![]);
    let Screen::Tu4(w) = &app.screen else {
        panic!("TU4 screen");
    };
    assert!(w.runner.as_ref().expect("runner").session.steps() > 1);
    app.dispatch(Command::Invoke(CommandId::Pause), &ctx)
        .expect("pause");
    let Screen::Tu4(w) = &app.screen else {
        panic!("TU4 screen");
    };
    assert_eq!(
        w.runner.as_ref().expect("runner").session.status(),
        RunStatus::Paused
    );
}

#[test]
fn notes_use_canvas_coordinates_and_support_selection_drag_delete_and_undo() {
    use crate::diagram::editor::{
        session::{SelectableKey, Zoom},
        tool::{EditTools, EditorTool},
    };
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let key = app.projects.insert(fixture(false));
    app.open(key).unwrap();
    let active = app.projects[key].document().entry_submachine();
    if let Screen::Diagram(w) = &mut app.screen {
        let viewport = &mut w.editor.canvas_mut(active).viewport;
        viewport.camera_zoom = Zoom::new(2.);
        viewport.camera_pan = LayoutPos::new(100., 50.);
    }
    app.dispatch(
        Command::SelectTool(EditorTool::Edit(EditTools::Comment)),
        &ctx,
    )
    .unwrap();
    frame(&mut app, &ctx, 1280., 820., 0., vec![]);
    pointer_click(&mut app, &ctx, egui::pos2(400., 350.), 0.1);
    let (note_key, position) = app.projects[key]
        .document()
        .submachine(active)
        .unwrap()
        .layout()
        .notes_iter()
        .map(|(key, note)| (key, note.position))
        .next()
        .unwrap();
    assert_eq!(position, LayoutPos::new(150., 150.));
    app.dispatch(Command::SelectTool(EditorTool::Edit(EditTools::Move)), &ctx)
        .unwrap();
    pointer_click(&mut app, &ctx, egui::pos2(420., 370.), 0.2);
    if let Screen::Diagram(w) = &mut app.screen {
        assert!(
            w.editor
                .canvas_mut(active)
                .interaction
                .selection_state
                .is_selected(&SelectableKey::Note(note_key))
        );
    }
    let pos = egui::pos2(420., 370.);
    frame(
        &mut app,
        &ctx,
        1280.,
        820.,
        0.3,
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    frame(
        &mut app,
        &ctx,
        1280.,
        820.,
        0.4,
        vec![egui::Event::PointerMoved(egui::pos2(570., 370.))],
    );
    frame(
        &mut app,
        &ctx,
        1280.,
        820.,
        0.5,
        vec![egui::Event::PointerButton {
            pos: egui::pos2(570., 370.),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert_eq!(
        app.projects[key]
            .document()
            .submachine(active)
            .unwrap()
            .layout()
            .notes_iter()
            .next()
            .unwrap()
            .1
            .position,
        LayoutPos::new(225., 150.)
    );
    app.dispatch(
        Command::SelectTool(EditorTool::Edit(EditTools::Delete)),
        &ctx,
    )
    .unwrap();
    pointer_click(&mut app, &ctx, egui::pos2(570., 370.), 0.6);
    assert_eq!(
        app.projects[key]
            .document()
            .submachine(active)
            .unwrap()
            .layout()
            .notes_iter()
            .count(),
        0
    );
    app.dispatch(Command::Invoke(CommandId::Undo), &ctx)
        .unwrap();
    assert_eq!(
        app.projects[key]
            .document()
            .submachine(active)
            .unwrap()
            .layout()
            .notes_iter()
            .count(),
        1
    );
}

#[test]
fn follow_execution_centers_each_step_and_notices_expire() {
    use crate::ui::test_renderer::text_rects;
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let key = app.projects.insert(fixture(false));
    app.open(key).unwrap();
    app.dispatch(Command::Invoke(CommandId::OpenRunner), &ctx)
        .unwrap();
    frame(&mut app, &ctx, 1280., 820., 0., vec![]);
    let out = frame(&mut app, &ctx, 1280., 820., 0.1, vec![]);
    let initial = text_rects(&out)
        .into_iter()
        .find(|(text, _)| text == crate::ui::theme::NODE_START)
        .unwrap()
        .1
        .center();
    app.dispatch(Command::Invoke(CommandId::Step), &ctx)
        .unwrap();
    let out = frame(&mut app, &ctx, 1280., 820., 0.2, vec![]);
    let write = text_rects(&out)
        .into_iter()
        .find(|(text, rect)| text == "1" && rect.height() > 20.)
        .unwrap()
        .1
        .center();
    assert!(
        initial.distance(write) < 1.,
        "the next node must occupy the same canvas center"
    );
    app.dispatch(Command::Invoke(CommandId::Step), &ctx)
        .unwrap();
    let out = frame(&mut app, &ctx, 1280., 820., 0.3, vec![]);
    let stop = text_rects(&out)
        .into_iter()
        .find(|(text, _)| text == crate::ui::theme::NODE_END)
        .unwrap()
        .1
        .center();
    assert!(initial.distance(stop) < 1.);
    app.set_notice(("Saved".into(), false));
    frame(&mut app, &ctx, 1280., 820., 1., vec![]);
    frame(&mut app, &ctx, 1280., 820., 5.9, vec![]);
    assert!(app.notice.is_some());
    frame(&mut app, &ctx, 1280., 820., 6.1, vec![]);
    assert!(app.notice.is_none());
}

fn key_press(key: egui::Key) -> Vec<egui::Event> {
    [true, false]
        .into_iter()
        .map(|pressed| egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        })
        .collect()
}

#[test]
fn arrows_move_selected_nodes_one_cell_and_history_restores_them() {
    use crate::diagram::editor::session::SelectableKey;
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let key = app.projects.insert(fixture(false));
    app.open(key).unwrap();
    let active = app.projects[key].document().entry_submachine();
    let nodes: Vec<_> = app.projects[key]
        .document()
        .submachine(active)
        .unwrap()
        .layout()
        .nodes_iter()
        .map(|(key, pos)| (key, *pos))
        .collect();
    if let Screen::Diagram(w) = &mut app.screen {
        w.editor
            .canvas_mut(active)
            .interaction
            .selection_state
            .selected = nodes[..2]
            .iter()
            .map(|(key, _)| SelectableKey::Node(*key))
            .collect();
    }
    frame(&mut app, &ctx, 1280., 820., 0., vec![]);
    for (i, arrow) in [egui::Key::ArrowRight, egui::Key::ArrowUp]
        .into_iter()
        .enumerate()
    {
        frame(
            &mut app,
            &ctx,
            1280.,
            820.,
            0.1 + i as f64 * 0.1,
            key_press(arrow),
        );
    }
    let layout = app.projects[key]
        .document()
        .submachine(active)
        .unwrap()
        .layout();
    for (node, pos) in &nodes[..2] {
        assert_eq!(
            layout.get_node_position(*node),
            Some(*pos + LayoutPos::new(75., -75.))
        );
    }
    assert_eq!(layout.get_node_position(nodes[2].0), Some(nodes[2].1));
    app.dispatch(Command::Invoke(CommandId::Undo), &ctx)
        .unwrap();
    app.dispatch(Command::Invoke(CommandId::Undo), &ctx)
        .unwrap();
    for (node, pos) in &nodes {
        assert_eq!(
            app.projects[key]
                .document()
                .submachine(active)
                .unwrap()
                .layout()
                .get_node_position(*node),
            Some(*pos)
        );
    }
}

#[test]
fn note_text_edit_keeps_its_center_and_supports_undo() {
    use crate::{diagram::editor::session::SelectableKey, ui::test_renderer::text_rects};
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let mut project = fixture(false);
    let active = project.document().entry_submachine();
    let mut machine = project.document().submachine(active).unwrap().clone();
    let note = machine
        .layout_mut()
        .add_note(LayoutPos::new(450., 450.), "Original note".into());
    project
        .execute(
            ProjectEdit::ReplaceMachine {
                key: active,
                machine: Box::new(machine),
            },
            None,
        )
        .unwrap();
    let key = app.projects.insert(project);
    app.open(key).unwrap();
    if let Screen::Diagram(w) = &mut app.screen {
        w.editor
            .canvas_mut(active)
            .interaction
            .selection_state
            .single_select(SelectableKey::Note(note));
    }
    frame(&mut app, &ctx, 1280., 820., 0., vec![]);
    let out = frame(&mut app, &ctx, 1280., 820., 0.1, vec![]);
    let fields: Vec<_> = text_rects(&out)
        .into_iter()
        .filter(|(text, _)| text == "Original note")
        .collect();
    assert_eq!(fields.len(), 2);
    let canvas = fields
        .iter()
        .find(|(_, r)| r.center().distance(egui::pos2(450., 450.)) < 1.)
        .unwrap();
    assert!(canvas.1.center().distance(egui::pos2(450., 450.)) < 1.);
    let edit = fields
        .iter()
        .find(|(_, r)| r.center().distance(egui::pos2(450., 450.)) > 1.)
        .unwrap()
        .1
        .center();
    pointer_click(&mut app, &ctx, edit, 0.2);
    frame(
        &mut app,
        &ctx,
        1280.,
        820.,
        0.3,
        vec![egui::Event::Text(" updated".into())],
    );
    let changed = app.projects[key]
        .document()
        .submachine(active)
        .unwrap()
        .layout()
        .notes_iter()
        .next()
        .unwrap()
        .1;
    assert!(changed.content.contains("updated"));
    assert_eq!(changed.position, LayoutPos::new(450., 450.));
    frame(
        &mut app,
        &ctx,
        1280.,
        820.,
        0.4,
        key_press(egui::Key::ArrowLeft),
    );
    app.dispatch(Command::Invoke(CommandId::Undo), &ctx)
        .unwrap();
    assert_eq!(
        app.projects[key]
            .document()
            .submachine(active)
            .unwrap()
            .layout()
            .notes_iter()
            .next()
            .unwrap()
            .1
            .content,
        "Original note"
    );
}

#[test]
fn project_deletion_requires_confirmation_and_persists_the_remaining_library() {
    use crate::ui::test_renderer::text_rects;
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let key = app.projects.insert(fixture(false));
    frame(&mut app, &ctx, 1280., 820., 0., vec![]);
    let out = frame(&mut app, &ctx, 1280., 820., 0.1, vec![]);
    let delete = text_rects(&out)
        .iter()
        .find(|(s, _)| s.trim() == "Delete project")
        .unwrap()
        .1
        .center();
    pointer_click(&mut app, &ctx, delete, 0.2);
    assert!(app.projects.contains_key(key));
    assert_eq!(app.menu.delete_project, Some(key));
    frame(
        &mut app,
        &ctx,
        1280.,
        820.,
        0.3,
        key_press(egui::Key::Escape),
    );
    assert!(app.projects.contains_key(key));
    assert!(app.menu.delete_project.is_none());
    frame(&mut app, &ctx, 1280., 820., 0.4, vec![]);
    pointer_click(&mut app, &ctx, delete, 0.5);
    frame(&mut app, &ctx, 1280., 820., 0.6, vec![]);
    let out = frame(&mut app, &ctx, 1280., 820., 0.7, vec![]);
    let confirm = text_rects(&out)
        .into_iter()
        .filter(|(s, _)| s.trim() == "Delete project")
        .last()
        .unwrap()
        .1
        .center();
    pointer_click(&mut app, &ctx, confirm, 0.8);
    assert!(!app.projects.contains_key(key));
    let mut storage = MemoryStorage::default();
    app.persist(&mut storage).unwrap();
    assert!(
        persistence::decode_library(&storage.0.get(persistence::STORAGE_KEY).unwrap())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn cheatsheet_opens_in_both_workspaces_and_blocks_tool_shortcuts() {
    use crate::ui::test_renderer::text_rects;
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let key = app.projects.insert(fixture(false));
    app.open(key).unwrap();
    frame(&mut app, &ctx, 1280., 820., 0., key_press(egui::Key::F1));
    assert!(app.cheatsheet_open);
    let out = frame(&mut app, &ctx, 1280., 820., 0.1, key_press(egui::Key::B));
    assert!(
        text_rects(&out)
            .iter()
            .any(|(text, _)| text == "Editor cheatsheet")
    );
    if let Screen::Diagram(w) = &app.screen {
        assert_eq!(*w.editor.tool(), Default::default());
    }
    frame(
        &mut app,
        &ctx,
        1280.,
        820.,
        0.2,
        key_press(egui::Key::Escape),
    );
    assert!(!app.cheatsheet_open);
    app.dispatch(Command::Invoke(CommandId::OpenRunner), &ctx)
        .unwrap();
    frame(&mut app, &ctx, 1280., 820., 0.3, key_press(egui::Key::F1));
    let out = frame(&mut app, &ctx, 1280., 820., 0.4, vec![]);
    assert!(
        text_rects(&out)
            .iter()
            .any(|(text, _)| text == "Runner cheatsheet")
    );
}

#[test]
fn delete_tool_selection_click_and_keyboard_deletion_support_undo() {
    use crate::{
        diagram::editor::{session::SelectableKey, tool::EditorTool},
        ui::test_renderer::text_rects,
    };
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let mut project = fixture(false);
    let active = project.document().entry_submachine();
    let mut machine = project.document().submachine(active).unwrap().clone();
    let node = machine.machine().nodes_iter().next().unwrap().0;
    let edge = machine.machine().edges_iter().last().unwrap().0;
    let note = machine
        .layout_mut()
        .add_note(LayoutPos::new(450., 450.), "Delete me".into());
    project
        .execute(
            ProjectEdit::ReplaceMachine {
                key: active,
                machine: Box::new(machine),
            },
            None,
        )
        .unwrap();
    let original = project.data().clone();
    let key = app.projects.insert(project);
    app.open(key).unwrap();
    let select = |app: &mut App| {
        if let Screen::Diagram(w) = &mut app.screen {
            w.editor
                .canvas_mut(active)
                .interaction
                .selection_state
                .selected = vec![
                SelectableKey::Node(node),
                SelectableKey::Edge(edge),
                SelectableKey::Note(note),
            ];
        }
    };
    select(&mut app);
    frame(&mut app, &ctx, 1280., 820., 0., vec![]);
    let out = frame(&mut app, &ctx, 1280., 820., 0.1, vec![]);
    let label = text_rects(&out)
        .into_iter()
        .find(|(text, _)| text == "Delete")
        .unwrap()
        .1;
    pointer_click(&mut app, &ctx, label.center() - egui::vec2(0., 40.), 0.2);
    assert!(
        app.projects[key].data() == &original,
        "choosing Delete preserves the selection"
    );
    assert!(
        matches!(&app.screen, Screen::Diagram(w) if *w.editor.tool() == EditorTool::Edit(crate::diagram::editor::tool::EditTools::Delete))
    );
    app.dispatch(Command::SelectTool(EditorTool::default()), &ctx)
        .unwrap();
    let out = frame(&mut app, &ctx, 1280., 820., 0.3, key_press(egui::Key::X));
    assert!(
        matches!(&app.screen, Screen::Diagram(w) if *w.editor.tool() == EditorTool::Edit(crate::diagram::editor::tool::EditTools::Delete))
    );
    assert!(
        app.projects[key].data() == &original,
        "X selects the tool without deleting"
    );
    let node_pos = text_rects(&out)
        .into_iter()
        .find(|(text, rect)| text == crate::ui::theme::NODE_START && rect.left() > 216.)
        .unwrap()
        .1
        .center();
    pointer_click(&mut app, &ctx, node_pos, 0.4);
    let machine = app.projects[key].document().submachine(active).unwrap();
    assert!(machine.machine().get_node(node).is_none());
    assert_eq!(machine.machine().edges_iter().count(), 1);
    assert_eq!(machine.layout().notes_iter().count(), 1);
    app.dispatch(Command::Invoke(CommandId::Undo), &ctx)
        .unwrap();
    assert!(app.projects[key].data() == &original);
    select(&mut app);
    frame(
        &mut app,
        &ctx,
        1280.,
        820.,
        0.5,
        key_press(egui::Key::Delete),
    );
    assert_eq!(
        app.projects[key]
            .document()
            .submachine(active)
            .unwrap()
            .layout()
            .notes_iter()
            .count(),
        0
    );
    app.dispatch(Command::Invoke(CommandId::Undo), &ctx)
        .unwrap();
    assert!(app.projects[key].data() == &original);
}

#[test]
fn node_hover_and_selection_override_problem_colors() {
    use crate::{diagram::editor::session::SelectableKey, ui::theme};
    fn node_rect(output: &egui::FullOutput) -> &egui::epaint::RectShape {
        output
            .shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rect)
                    if rect.rect.center().distance(egui::pos2(450., 225.)) < 0.1 =>
                {
                    Some(rect)
                }
                _ => None,
            })
            .expect("node rectangle")
    }
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let mut document = crate::diagram::model::document::DiagramDocument::new(
        ValidatedAlphabet::new([' ', '0']).unwrap(),
    );
    let active = document.entry_submachine();
    let node = document
        .submachine_mut(active)
        .unwrap()
        .add_node(LayoutPos::new(450., 225.), NodeAction::Start);
    let key = app.projects.insert(Project::new(
        ProjectName::new("Colors".into()).unwrap(),
        document,
    ));
    app.open(key).unwrap();
    frame(&mut app, &ctx, 1280., 820., 0., vec![]);
    let hover = frame(
        &mut app,
        &ctx,
        1280.,
        820.,
        0.1,
        vec![egui::Event::PointerMoved(egui::pos2(450., 225.))],
    );
    assert_eq!(node_rect(&hover).stroke.color, theme::SELECTED_NODE_STROKE);
    assert_ne!(node_rect(&hover).fill, theme::PROBLEM_NODE_FILL);
    if let Screen::Diagram(w) = &mut app.screen {
        w.editor
            .canvas_mut(active)
            .interaction
            .selection_state
            .single_select(SelectableKey::Node(node));
    }
    let selected = frame(&mut app, &ctx, 1280., 820., 0.2, vec![]);
    assert_eq!(
        node_rect(&selected).stroke.color,
        theme::SELECTED_NODE_STROKE
    );
    assert_eq!(node_rect(&selected).fill, theme::SELECTED_NODE_FILL);
}

#[test]
fn runner_cards_navigate_and_tape_text_controls_stay_on_one_row() {
    use crate::ui::test_renderer::text_rects;
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let mut document = fixture(false).document().clone();
    let child = document.add_submachine(SubmachineName::parse("Child machine".into()).unwrap());
    let alphabet = document.alphabet().clone();
    let machine = document.submachine_mut(child).unwrap();
    let start = machine.add_node(LayoutPos::new(450., 225.), NodeAction::Start);
    let stop = machine.add_node(LayoutPos::new(600., 225.), NodeAction::Stop);
    machine
        .add_edge(
            TransitionSymbols::new(alphabet.iter().copied(), &alphabet).unwrap(),
            start,
            stop,
        )
        .unwrap();
    let key = app.projects.insert(Project::new(
        ProjectName::new("Navigation".into()).unwrap(),
        document,
    ));
    app.open(key).unwrap();
    app.dispatch(Command::Invoke(CommandId::OpenRunner), &ctx)
        .unwrap();
    frame(&mut app, &ctx, 1280., 1000., 0., vec![]);
    let out = frame(&mut app, &ctx, 1280., 1000., 0.1, vec![]);
    let labels = text_rects(&out);
    let rect = |label: &str| labels.iter().find(|(text, _)| text == label).unwrap().1;
    assert!((rect("Apply tape").center().y - rect("Edit tape as text").center().y).abs() < 3.);
    assert!(rect("Follow step execution").top() > rect("Step interval").bottom());
    assert!(rect("Follow step execution").bottom() < rect("Status: Paused").top());
    let card = rect("Child machine").center();
    pointer_click(&mut app, &ctx, card, 0.2);
    if let Screen::Diagram(w) = &app.screen {
        assert_eq!(w.runner.as_ref().unwrap().ui.active_submachine, child);
    }
    frame(&mut app, &ctx, 1280., 1000., 0.3, vec![]);
    if let Screen::Diagram(w) = &app.screen {
        assert_eq!(w.runner.as_ref().unwrap().ui.active_submachine, child);
    }
}

#[test]
fn dragging_a_note_renders_a_preview_at_its_snapped_center() {
    use crate::diagram::editor::session::SelectableKey;
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let mut project = fixture(false);
    let active = project.document().entry_submachine();
    let mut machine = project.document().submachine(active).unwrap().clone();
    let note = machine
        .layout_mut()
        .add_note(LayoutPos::new(480., 450.), "Preview".into());
    project
        .execute(
            ProjectEdit::ReplaceMachine {
                key: active,
                machine: Box::new(machine),
            },
            None,
        )
        .unwrap();
    let key = app.projects.insert(project);
    app.open(key).unwrap();
    if let Screen::Diagram(w) = &mut app.screen {
        w.editor.canvas_mut(active).interaction.drag = vec![SelectableKey::Note(note)];
    }
    let out = frame(&mut app, &ctx, 1280., 820., 0., vec![]);
    assert!(out.shapes.iter().any(|clipped| matches!(&clipped.shape,
        egui::Shape::Rect(rect) if rect.fill == egui::Color32::from_rgba_unmultiplied(0, 0, 0, 50)
            && rect.rect.center().distance(egui::pos2(450., 450.)) < 0.1)));
}

#[test]
fn cheatsheet_uses_a_grouped_table_with_screen_insets() {
    use crate::ui::test_renderer::{TestRenderer, text_rects};
    for runner in [false, true] {
        let ctx = egui::Context::default();
        ctx.set_visuals(crate::ui::theme::setup_visuals());
        let mut renderer = TestRenderer::default();
        let mut open = true;
        for index in 0..3 {
            let out = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1280., 820.),
                    )),
                    time: Some(index as f64 * 0.1),
                    ..Default::default()
                },
                |ctx| crate::ui::cheatsheet::show(ctx, &mut open, runner),
            );
            renderer.update(&out);
            if index == 2 {
                let labels = text_rects(&out);
                let rect = |name: &str| labels.iter().find(|(text, _)| text == name).unwrap().1;
                assert!(rect("Group").center().x < rect("Key / mouse").center().x);
                assert!(rect("Key / mouse").center().x < rect("Action").center().x);
                assert!((rect("Group").center().y - rect("Action").center().y).abs() < 2.);
                assert!(rect("Group").left() < 250.);
                assert!(rect("Action").right() > 900.);
                assert!(labels.iter().any(|(text, _)| text == "Project"));
                if let Some(dir) = std::env::var_os("TMTINKER_UI_SNAPSHOTS") {
                    let dir = std::path::PathBuf::from(dir);
                    std::fs::create_dir_all(&dir).unwrap();
                    renderer.save(
                        &ctx,
                        out,
                        1280,
                        820,
                        &dir.join(if runner {
                            "runner-cheatsheet.ppm"
                        } else {
                            "editor-cheatsheet.ppm"
                        }),
                    );
                }
            }
        }
    }
}

#[test]
fn runner_draws_the_next_transition_and_clears_it_when_stopped() {
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let key = app.projects.insert(fixture(false));
    app.open(key).unwrap();
    app.dispatch(Command::Invoke(CommandId::OpenRunner), &ctx)
        .unwrap();
    frame(&mut app, &ctx, 1280., 820., 0., vec![]);
    let out = frame(&mut app, &ctx, 1280., 820., 0.1, vec![]);
    let highlighted_lines = |out: &egui::FullOutput| {
        out.shapes.iter().filter(|clipped| matches!(
        &clipped.shape, egui::Shape::LineSegment { stroke, .. }
            if stroke.color == crate::ui::theme::ACTIVE_NODE_STROKE && stroke.width == 3.5
    )).count()
    };
    assert!(highlighted_lines(&out) > 0);
    assert!(out.shapes.iter().any(|clipped| matches!(&clipped.shape,
        egui::Shape::Path(path) if path.fill == crate::ui::theme::ACTIVE_NODE_STROKE)));
    app.dispatch(Command::Invoke(CommandId::Step), &ctx)
        .unwrap();
    let out = frame(&mut app, &ctx, 1280., 820., 0.2, vec![]);
    assert!(highlighted_lines(&out) > 0);
    app.dispatch(Command::Invoke(CommandId::Stop), &ctx)
        .unwrap();
    let out = frame(&mut app, &ctx, 1280., 820., 0.3, vec![]);
    assert_eq!(highlighted_lines(&out), 0);
}

fn shortcut_commands(
    app: &App,
    ctx: &egui::Context,
    key: egui::Key,
    modifiers: egui::Modifiers,
    typing: bool,
) -> Vec<CommandId> {
    let mut commands = Vec::new();
    let mut text = String::from("copy me");
    let _ = ctx.run(
        egui::RawInput {
            modifiers,
            events: [true, false]
                .into_iter()
                .map(|pressed| egui::Event::Key {
                    key,
                    physical_key: Some(key),
                    pressed,
                    repeat: false,
                    modifiers,
                })
                .collect(),
            ..Default::default()
        },
        |ctx| {
            if typing {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.text_edit_singleline(&mut text).request_focus();
                });
            }
            app.collect_shortcuts(ctx, &mut commands);
        },
    );
    commands.into_iter().map(|command| command.id()).collect()
}

#[test]
fn workspace_shortcuts_follow_the_current_mode_and_runner_status() {
    use egui::{Key, Modifiers};
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let key = app.projects.insert(fixture(false));
    assert_eq!(
        shortcut_commands(&app, &ctx, Key::N, Modifiers::COMMAND, false),
        [CommandId::NewProject]
    );
    app.open(key).unwrap();
    for (key, modifiers, expected) in [
        (Key::N, Modifiers::COMMAND, CommandId::AddMachine),
        (Key::V, Modifiers::NONE, CommandId::Validate),
        (Key::M, Modifiers::NONE, CommandId::Move),
        (Key::P, Modifiers::NONE, CommandId::OpenRunner),
    ] {
        assert_eq!(
            shortcut_commands(&app, &ctx, key, modifiers, false),
            [expected]
        );
    }
    app.dispatch(Command::Invoke(CommandId::AddMachine), &ctx)
        .unwrap();
    assert!(matches!(&app.screen, Screen::Diagram(w) if w.ui.create_open));
    assert!(shortcut_commands(&app, &ctx, Key::P, Modifiers::NONE, false).is_empty());
    if let Screen::Diagram(w) = &mut app.screen {
        w.ui.create_open = false;
    }
    app.dispatch(Command::Invoke(CommandId::OpenRunner), &ctx)
        .unwrap();
    for id in [
        CommandId::Run,
        CommandId::Pause,
        CommandId::Step,
        CommandId::Reset,
        CommandId::Stop,
    ] {
        let info = id.info();
        let modifiers = if info.modified {
            Modifiers::COMMAND
        } else {
            Modifiers::NONE
        };
        assert_eq!(
            shortcut_commands(&app, &ctx, info.key.unwrap(), modifiers, false),
            [id]
        );
        app.dispatch(Command::Invoke(id), &ctx).unwrap();
    }
    assert!(
        matches!(&app.screen, Screen::Diagram(w) if w.runner.as_ref().unwrap().session.status() == RunStatus::Halted)
    );
    for id in [CommandId::Undo, CommandId::Redo, CommandId::AddMachine] {
        assert!(!app.enabled(id));
        let info = id.info();
        assert!(
            shortcut_commands(&app, &ctx, info.key.unwrap(), Modifiers::COMMAND, false).is_empty()
        );
    }
    assert!(
        shortcut_commands(
            &app,
            &ctx,
            Key::Z,
            Modifiers::COMMAND | Modifiers::SHIFT,
            false
        )
        .is_empty()
    );
    app.dispatch(Command::Invoke(CommandId::Reset), &ctx)
        .unwrap();
    for (key, modifiers) in [
        (Key::P, Modifiers::NONE),
        (Key::Space, Modifiers::NONE),
        (Key::C, Modifiers::COMMAND),
        (Key::D, Modifiers::COMMAND),
    ] {
        assert!(shortcut_commands(&app, &ctx, key, modifiers, true).is_empty());
    }
}

#[test]
fn canvas_pans_with_both_touchpad_axes_and_shift_wheel() {
    use crate::diagram::editor::session::Zoom;
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    let key = app.projects.insert(fixture(false));
    let active = app.projects[key].document().entry_submachine();
    app.open(key).unwrap();
    frame(&mut app, &ctx, 1280., 820., 0., vec![]);
    if let Screen::Diagram(w) = &mut app.screen {
        w.editor.canvas_mut(active).viewport.camera_zoom = Zoom::new(2.0);
    }
    let pan = |app: &mut App| match &mut app.screen {
        Screen::Diagram(w) => w.editor.canvas_mut(active).viewport.camera_pan,
        _ => panic!("editor is open"),
    };
    let cases = [
        (
            egui::vec2(4., 0.),
            egui::Modifiers::NONE,
            egui::vec2(4., 0.),
        ),
        (
            egui::vec2(0., 4.),
            egui::Modifiers::NONE,
            egui::vec2(0., 4.),
        ),
        (
            egui::vec2(3., -4.),
            egui::Modifiers::NONE,
            egui::vec2(3., -4.),
        ),
        (
            egui::vec2(0., 4.),
            egui::Modifiers::SHIFT,
            egui::vec2(4., 0.),
        ),
    ];
    for (index, (delta, modifiers, expected)) in cases.into_iter().enumerate() {
        let before = pan(&mut app);
        frame(
            &mut app,
            &ctx,
            1280.,
            820.,
            0.1 + index as f64 * 0.1,
            vec![
                egui::Event::PointerMoved(egui::pos2(900., 700.)),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta,
                    modifiers,
                },
            ],
        );
        let after = pan(&mut app);
        assert_eq!(after.x - before.x, expected.x, "horizontal screen movement");
        assert_eq!(after.y - before.y, expected.y, "vertical screen movement");
    }
    let before = pan(&mut app);
    frame(
        &mut app,
        &ctx,
        1280.,
        820.,
        0.6,
        vec![
            egui::Event::PointerMoved(egui::pos2(50., 700.)),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Point,
                delta: egui::vec2(4., 3.),
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    assert_eq!(
        pan(&mut app),
        before,
        "scrolling a sidebar does not pan the canvas"
    );
}

#[test]
fn tu4_gutter_toggles_breakpoints_only_in_runner() {
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    app.dispatch(
        Command::CreateTu4 {
            name: ProjectName::new("Debug".into()).unwrap(),
            alphabet: ValidatedAlphabet::new([' ', '1']).unwrap(),
        },
        &ctx,
    )
    .unwrap();
    app.dispatch(
        Command::Edit {
            edit: ProjectEdit::SetSource(std::sync::Arc::from("0,λ,>,1\n1,λ,~,0\n")),
            group: None,
        },
        &ctx,
    )
    .unwrap();
    let circle = |out: &egui::FullOutput| {
        out.shapes.iter().find_map(|shape| match &shape.shape {
            egui::Shape::Circle(circle)
                if circle.radius == 5. && circle.stroke.color == egui::Color32::GRAY =>
            {
                Some(circle.center)
            }
            _ => None,
        })
    };
    let out = frame(&mut app, &ctx, 1280., 820., 0.1, vec![]);
    assert!(circle(&out).is_none(), "editor has no breakpoint circles");
    app.dispatch(Command::Invoke(CommandId::OpenRunner), &ctx)
        .unwrap();
    let out = frame(&mut app, &ctx, 1280., 820., 0.2, vec![]);
    pointer_click(
        &mut app,
        &ctx,
        circle(&out).expect("runner breakpoint"),
        0.21,
    );
    assert!(
        app.project()
            .unwrap()
            .tu4()
            .unwrap()
            .breakpoints
            .contains(&1)
    );
    let out = frame(&mut app, &ctx, 1280., 820., 0.3, vec![]);
    pointer_click(
        &mut app,
        &ctx,
        circle(&out).expect("second breakpoint"),
        0.4,
    );
    assert!(
        app.project()
            .unwrap()
            .tu4()
            .unwrap()
            .breakpoints
            .contains(&2)
    );
    app.dispatch(Command::Invoke(CommandId::Run), &ctx).unwrap();
    frame(&mut app, &ctx, 1280., 820., 10., vec![]);
    let Screen::Tu4(w) = &app.screen else {
        panic!("TU4 screen");
    };
    let session = &w.runner.as_ref().unwrap().session;
    assert_eq!(session.status(), RunStatus::Paused);
    assert_eq!(session.steps(), 0);
}

#[test]
fn tu4_follow_keeps_the_last_command_stable_with_space_below() {
    use crate::ui::test_renderer::text_rects;
    let ctx = egui::Context::default();
    let mut app = App::new(&eframe::CreationContext::_new_kittest(ctx.clone()));
    app.dispatch(
        Command::CreateTu4 {
            name: ProjectName::new("Scroll".into()).unwrap(),
            alphabet: ValidatedAlphabet::new([' ', '1']).unwrap(),
        },
        &ctx,
    )
    .unwrap();
    let mut source = (0..99)
        .map(|state| format!("{state},λ,>,{}\n", state + 1))
        .collect::<String>();
    source.push_str("99,λ,~,0");
    app.dispatch(
        Command::Edit {
            edit: ProjectEdit::SetSource(source.into()),
            group: None,
        },
        &ctx,
    )
    .unwrap();
    app.dispatch(Command::Invoke(CommandId::OpenRunner), &ctx)
        .unwrap();
    frame(&mut app, &ctx, 1280., 820., 0.1, vec![]);
    for _ in 0..99 {
        app.dispatch(Command::Invoke(CommandId::Step), &ctx)
            .unwrap();
    }
    let mut positions = Vec::new();
    for tick in 0..5 {
        let out = frame(&mut app, &ctx, 1280., 820., 1. + tick as f64, vec![]);
        positions.push(
            text_rects(&out)
                .iter()
                .find(|(text, _)| text == "99,λ,~,0")
                .expect("last command visible")
                .1
                .center()
                .y,
        );
    }
    for y in &positions {
        assert!(
            *y > 100. && *y < 500.,
            "last command should have room below: {positions:?}"
        );
        assert!(
            (y - positions[0]).abs() < 1.,
            "scroll position changed: {positions:?}"
        );
    }
}
