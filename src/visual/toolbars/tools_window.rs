use crate::{
    core::tm::SubmachineKey, project::Project, visual::{
        mappings::{
            BUTTON_CORNER_RADIUS, BUTTON_SIZE, BUTTON_STROKE_WIDTH, ICON_FONT_SIZE, SELECTED_NODE_STROKE, SUBSCRIPT_FONT_SIZE, TOOL_DELETE, TOOL_END, TOOL_LEFT, TOOL_LINK, TOOL_MOVE, TOOL_PLACE_SIGN, TOOL_RIGHT, TOOL_START,
        }, router::ToolState, tools::dyn_tools_base::{BasicActionTools, EditTools, EditorTool}, utils::shortcuts::*,
    },
};
use egui::{Context, Id, RichText, TextBuffer, Ui, vec2};

#[derive(Default, Clone)]
pub struct SubmachineCreationWindow {
    pub show_window: bool,
    pub name_buffer: String,
}

fn tool_button(
    ui: &mut egui::Ui,
    tool: EditorTool,
    icon: &str,
    label: &str,
    shortcut: Option<&str>,
    tool_state: &mut ToolState,
) {
    let is_selected = tool_state.current_tool == tool;
    let button_size = egui::vec2(BUTTON_SIZE, BUTTON_SIZE);

    ui.allocate_ui_with_layout(
        egui::vec2(button_size.x, button_size.y + 25.0),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            let response =
                ui.add_sized(
                    button_size,
                    egui::Button::new(RichText::new(icon).size(ICON_FONT_SIZE).color(
                        if is_selected {
                            SELECTED_NODE_STROKE
                        } else {
                            ui.visuals().text_color()
                        },
                    ))
                    .frame(true),
                );

            if response.clicked() {
                tool_state.current_tool = tool;
            }

            if let Some(shortcut) = shortcut {
                let rect = response.rect;

                let galley = ui.painter().layout_no_wrap(
                    shortcut.to_owned(),
                    egui::FontId::proportional(10.0),
                    ui.visuals().weak_text_color(),
                );

                let padding = 3.0;
                let text_pos = egui::pos2(
                    rect.right() - galley.size().x - padding,
                    rect.top() + padding,
                );

                ui.painter()
                    .galley(text_pos, galley, ui.visuals().weak_text_color());
            }

            ui.label(RichText::new(label).size(SUBSCRIPT_FONT_SIZE));

            if is_selected {
                let rect = response.rect.expand2(egui::vec2(2.0, 2.0));
                ui.painter().rect_stroke(
                    rect,
                    BUTTON_CORNER_RADIUS,
                    egui::Stroke::new(BUTTON_STROKE_WIDTH, SELECTED_NODE_STROKE),
                    egui::StrokeKind::Inside,
                );
            }
        },
    );
}

fn submachine_button(
    ui: &mut egui::Ui,
    key: SubmachineKey,
    machine_name: &str,
    tool_state: &mut ToolState,
) -> Option<SubmachineKey> {
    let mut buf = [0u8; 4];
    let mut should_create = false;

    ui.horizontal(|ui| {
        tool_button(
            ui,
            EditorTool::Submachine {
                name: machine_name.to_owned(),
                key: key,
                power: 1,
            },
            machine_name
                .chars()
                .next()
                .expect("Name is validated")
                .encode_utf8(&mut buf),
            machine_name,
            Some(text_space_node_shortcut()),
            tool_state,
        );

        let response = ui
            .add(
                egui::Button::new(
                    RichText::new("👁")
                        .size(ICON_FONT_SIZE * 0.7),
                )
                .frame(true),
            )
            .on_hover_text("Open submachine");

        if response.clicked() {
            should_create = true;
        }
    });

    if should_create {
        Some(key)
    } else {
        None
    }
}

fn add_submachine_button(
    project: &mut Project,
    ui: &mut egui::Ui,
    ctx: &Context,
    shortcut: Option<&str>,
) {
    let button_size = egui::vec2(BUTTON_SIZE, BUTTON_SIZE);

    let form_id = Id::new("submachine_creation_window");

    let mut form = ui.data_mut(|mem| {
        mem.get_temp::<SubmachineCreationWindow>(form_id).unwrap_or_default()
    });

    ui.allocate_ui_with_layout(
        egui::vec2(button_size.x, button_size.y + 25.0),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            let response =
                ui.add_sized(
                    button_size,
                    egui::Button::new(RichText::new("+").size(ICON_FONT_SIZE).color(
                            ui.visuals().text_color()
                    ))
                    .frame(true),
                );

            if response.clicked() {
                form.show_window = true;
            }

            let mut should_create = false;

            egui::Window::new("Create submachine")
                .open(&mut form.show_window)
                .show(ctx, |ui| {
                    ui.text_edit_singleline(&mut form.name_buffer);
                    if ui.button("Create submachine").clicked() {
                        should_create = true; // Только фиксируем клик
                    }
                });

            if should_create {
                project.add_submachine(form.name_buffer.take());
                form.show_window = false;
            }

            if let Some(shortcut) = shortcut {
                let rect = response.rect;

                let galley = ui.painter().layout_no_wrap(
                    shortcut.to_owned(),
                    egui::FontId::proportional(10.0),
                    ui.visuals().weak_text_color(),
                );

                let padding = 3.0;
                let text_pos = egui::pos2(
                    rect.right() - galley.size().x - padding,
                    rect.top() + padding,
                );

                ui.painter()
                    .galley(text_pos, galley, ui.visuals().weak_text_color());
            }

            ui.label(RichText::new("New submachine").size(SUBSCRIPT_FONT_SIZE));
        },
    );

    ui.data_mut(|mem| {
        mem.insert_temp(form_id, form);
    });
}

pub fn select_tool(project: &mut Project, ui: &mut Ui, tool_state: &mut ToolState, ctx: &Context) {
    egui::ScrollArea::both().show(ui, |ui| {
        egui::Grid::new("Select tool basic")
            .num_columns(2)
            .striped(false)
            .spacing(vec2(15.0, 10.0))
            .show(ui, |ui| {
                tool_button(
                    ui,
                    EditorTool::Edit(EditTools::Move),
                    TOOL_MOVE,
                    "Move",
                    Some(text_move_node_shortcut()),
                    tool_state,
                );
                tool_button(
                    ui,
                    EditorTool::Edit(EditTools::Link),
                    TOOL_LINK,
                    "Link",
                    Some(text_move_node_shortcut()),
                    tool_state,
                );
                ui.end_row();
                tool_button(
                    ui,
                    EditorTool::Edit(EditTools::Comment),
                    TOOL_DELETE,
                    "Note",
                    Some(text_delete_node_shortcut()),
                    tool_state,
                );
                tool_button(
                    ui,
                    EditorTool::Edit(EditTools::Delete),
                    TOOL_DELETE,
                    "Delete",
                    Some(text_delete_node_shortcut()),
                    tool_state,
                );
                ui.end_row();
                ui.label("Basic actions");
                ui.end_row();
                tool_button(
                    ui,
                    EditorTool::Place(BasicActionTools::Start),
                    TOOL_START,
                    "Start",
                    Some(text_start_node_shortcut()),
                    tool_state,
                );
                tool_button(
                    ui,
                    EditorTool::Place(BasicActionTools::Stop),
                    TOOL_END,
                    "End",
                    Some(text_end_node_shortcut()),
                    tool_state,
                );
                ui.end_row();
                tool_button(
                    ui,
                    EditorTool::Place(BasicActionTools::Left),
                    TOOL_LEFT,
                    "Left",
                    Some(text_left_node_shortcut()),
                    tool_state,
                );
                tool_button(
                    ui,
                    EditorTool::Place(BasicActionTools::Right),
                    TOOL_RIGHT,
                    "Right",
                    Some(text_right_node_shortcut()),
                    tool_state,
                );
                ui.end_row();
                tool_button(
                    ui,
                    EditorTool::Place(BasicActionTools::Write),
                    TOOL_PLACE_SIGN,
                    "Write",
                    Some(text_space_node_shortcut()),
                    tool_state,
                );
                ui.end_row();
                ui.label("Submachines");
                ui.end_row();

                let mut next_submachine = None;

                for (key, document) in project.list_submachines() {
                    if key == *project.current_submachine_key() {
                        continue;
                    }

                    let machine_name = document.machine.name().to_owned();

                    next_submachine = submachine_button(
                        ui,
                        key,
                        &machine_name,
                        tool_state,
                    );

                    ui.end_row();
                }

                if let Some(key) = next_submachine {
                    project.set_submachine(key);
                }

                add_submachine_button(project, ui, ctx, None);

                // TODO add dedicated button with settings
            });
    });
}
