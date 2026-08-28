#![allow(dead_code)]
use egui::{
    Color32, Stroke, Visuals, style::{WidgetVisuals, Widgets},
};

macro_rules! rgb_from_hex {
    ($hex:expr) => {{
        const HEX: &[u8] = $hex;

        #[expect(clippy::indexing_slicing)]
        const _: () = assert!(HEX[0] == b'#', "Hex color must start with '#'");
        const _: () = assert!(HEX.len() == 7, "Hex color must be 7 bytes \"#******\"");

        const fn hex_digit(b: u8) -> u8 {
            match b {
                b'0'..=b'9' => b - b'0',
                b'a'..=b'f' => b - b'a' + 10,
                b'A'..=b'F' => b - b'A' + 10,
                _ => panic!("Invalid hex digit"),
            }
        }

        #[expect(clippy::indexing_slicing)]
        const R: u8 = (hex_digit(HEX[1]) << 4) | hex_digit(HEX[2]);
        #[expect(clippy::indexing_slicing)]
        const G: u8 = (hex_digit(HEX[3]) << 4) | hex_digit(HEX[4]);
        #[expect(clippy::indexing_slicing)]
        const B: u8 = (hex_digit(HEX[5]) << 4) | hex_digit(HEX[6]);

        egui::Color32::from_rgb(R, G, B)
    }};
}

// // --- Node fills

// pub const SELECTED_NODE_FILL: Color32 = rgb_from_hex!(b"#45D8C7");
// pub const SELECTED_NODE_STROKE: Color32 = rgb_from_hex!(b"#00FFFF");

// pub const NOT_AVAILABLE_NODE_FILL: Color32 = rgb_from_hex!(b"#B52426");
// pub const NOT_AVAILABLE_NODE_STROKE: Color32 = rgb_from_hex!(b"#E02124");

// pub const ACTIVE_NODE_FILL: Color32 = rgb_from_hex!(b"#93DB0E");
// pub const ACTIVE_NODE_STROKE: Color32 = rgb_from_hex!(b"#A6FF00");

// pub const IDLE_NODE_FILL: Color32 = rgb_from_hex!(b"#1f2125");
// pub const IDLE_NODE_STROKE: Color32 = rgb_from_hex!(b"#41484a");

// pub const PROBLEM_NODE_FILL: Color32 = rgb_from_hex!(b"#D5C707");
// pub const PROBLEM_NODE_STROKE: Color32 = rgb_from_hex!(b"#FFE200");

// /// "Regular" node style — used for connection lines and neutral elements.
// pub const REGULAR_NODE_FILL: Color32 = rgb_from_hex!(b"#1f2125");
// pub const REGULAR_NODE_STROKE: Color32 = rgb_from_hex!(b"#41484a");

// /// "Working" node style — the node currently being executed.
// pub const WORKING_NODE_FILL: Color32 = rgb_from_hex!(b"#45D8C7");
// pub const WORKING_NODE_STROKE: Color32 = rgb_from_hex!(b"#00FFE1");

// // --- Canvas

// pub const CANVAS_BACKGROUND: Color32 = rgb_from_hex!(b"#161819");
// pub const CANVAS_GRID: Color32 = rgb_from_hex!(b"#1e2021");

// // --- Panels

// pub const PANEL_BACKGROUND: Color32 = rgb_from_hex!(b"#1f2125");
// pub const PANEL_STROKE: Color32 = rgb_from_hex!(b"#41484a");
// pub const WINDOW_FILL: Color32 = PANEL_BACKGROUND;

// pub const BUTTON_FILL: Color32 = rgb_from_hex!(b"#1f2125");
// pub const TEXT_COLOR: Color32 = rgb_from_hex!(b"#FFFFFF");

// // --- Sizes

// pub const SUBSCRIPT_FONT_SIZE: f32 = 14.0;
// pub const ICON_FONT_SIZE: f32 = 24.0;
// pub const BUTTON_SIZE: f32 = 60.0;
// pub const NODE_SIZE: f32 = 60.0;
// pub const BUTTON_HORIZONTAL_SPACING: f32 = 25.0;
// pub const BUTTON_VERTICAL_SPACING: f32 = 25.0;
// pub const BUTTON_FROM_BORDER_PADDING: f32 = 15.0;
// pub const NODE_STROKE_WIDTH: f32 = 1.5;
// pub const BUTTON_STROKE_WIDTH: f32 = NODE_STROKE_WIDTH;
// pub const BUTTON_CORNER_RADIUS: f32 = 4.0;

// // --- Widget theme

// pub fn dark_theme_widgets() -> Widgets {
//     Widgets {
//         noninteractive: dark_widget_noninteractive(),
//         inactive: dark_widget_inactive(),
//         hovered: dark_widget_hovered(),
//         active: dark_widget_active(),
//         open: dark_widget_open(),
//     }
// }

// fn dark_widget_noninteractive() -> WidgetVisuals {
//     WidgetVisuals {
//         bg_fill: PANEL_BACKGROUND,
//         weak_bg_fill: BUTTON_FILL,
//         bg_stroke: Stroke::new(BUTTON_STROKE_WIDTH, PANEL_STROKE),
//         fg_stroke: Stroke::new(BUTTON_STROKE_WIDTH, Color32::from_gray(180)),
//         corner_radius: BUTTON_CORNER_RADIUS.into(),
//         expansion: 0.0,
//     }
// }

// fn dark_widget_inactive() -> WidgetVisuals {
//     WidgetVisuals {
//         bg_fill: BUTTON_FILL,
//         weak_bg_fill: BUTTON_FILL,
//         bg_stroke: Stroke::new(BUTTON_STROKE_WIDTH, PANEL_STROKE),
//         fg_stroke: Stroke::NONE,
//         corner_radius: BUTTON_CORNER_RADIUS.into(),
//         expansion: 0.0,
//     }
// }

// fn dark_widget_hovered() -> WidgetVisuals {
//     let hovered_fill = Color32::from_rgb(
//         ((BUTTON_FILL.r() as u16 + 20).min(255)) as u8,
//         ((BUTTON_FILL.g() as u16 + 20).min(255)) as u8,
//         ((BUTTON_FILL.b() as u16 + 20).min(255)) as u8,
//     );
//     WidgetVisuals {
//         bg_fill: hovered_fill,
//         weak_bg_fill: BUTTON_FILL,
//         bg_stroke: Stroke::new(BUTTON_STROKE_WIDTH, SELECTED_NODE_STROKE),
//         fg_stroke: Stroke::new(BUTTON_STROKE_WIDTH, Color32::WHITE),
//         corner_radius: BUTTON_CORNER_RADIUS.into(),
//         expansion: 1.0,
//     }
// }

// fn dark_widget_active() -> WidgetVisuals {
//     WidgetVisuals {
//         bg_fill: SELECTED_NODE_FILL,
//         weak_bg_fill: SELECTED_NODE_FILL,
//         bg_stroke: Stroke::new(BUTTON_STROKE_WIDTH, SELECTED_NODE_STROKE),
//         fg_stroke: Stroke::new(BUTTON_STROKE_WIDTH, Color32::BLACK),
//         corner_radius: BUTTON_CORNER_RADIUS.into(),
//         expansion: 1.0,
//     }
// }

// fn dark_widget_open() -> WidgetVisuals {
//     WidgetVisuals {
//         bg_fill: BUTTON_FILL,
//         weak_bg_fill: BUTTON_FILL,
//         bg_stroke: Stroke::new(BUTTON_STROKE_WIDTH, WORKING_NODE_STROKE),
//         fg_stroke: Stroke::new(BUTTON_STROKE_WIDTH, Color32::WHITE),
//         corner_radius: BUTTON_CORNER_RADIUS.into(),
//         expansion: 0.0,
//     }
// }

pub const SELECTED_NODE_FILL: Color32 = rgb_from_hex!(b"#45D8C7");
pub const SELECTED_NODE_STROKE: Color32 = rgb_from_hex!(b"#00B8A8");

pub const NOT_AVAILABLE_NODE_FILL: Color32 = rgb_from_hex!(b"#FFD84D");
pub const NOT_AVAILABLE_NODE_STROKE: Color32 = rgb_from_hex!(b"#D4A807");

pub const ACTIVE_NODE_FILL: Color32 = rgb_from_hex!(b"#93DB0E");
pub const ACTIVE_NODE_STROKE: Color32 = rgb_from_hex!(b"#6FB000");

// pub const IDLE_NODE_FILL: Color32 = rgb_from_hex!(b"#F0F2F5");
pub const IDLE_NODE_FILL: Color32 = rgb_from_hex!(b"#e1e3e6");
pub const IDLE_NODE_STROKE: Color32 = rgb_from_hex!(b"#B0B5BA");

pub const PROBLEM_NODE_FILL: Color32 = rgb_from_hex!(b"#E85A5C");
pub const PROBLEM_NODE_STROKE: Color32 = rgb_from_hex!(b"#C02124");

/// "Regular" node style — used for connection lines and neutral elements.
pub const REGULAR_NODE_FILL: Color32 = rgb_from_hex!(b"#F0F2F5");
pub const REGULAR_NODE_STROKE: Color32 = rgb_from_hex!(b"#B0B5BA");

/// "Working" node style — the node currently being executed.
pub const WORKING_NODE_FILL: Color32 = rgb_from_hex!(b"#45D8C7");
pub const WORKING_NODE_STROKE: Color32 = rgb_from_hex!(b"#00B8A8");

// --- Canvas

pub const CANVAS_BACKGROUND: Color32 = rgb_from_hex!(b"#FFFFFF");
pub const CANVAS_GRID: Color32 = rgb_from_hex!(b"#E8EAED");

// --- Panels

pub const PANEL_BACKGROUND: Color32 = rgb_from_hex!(b"#FFFFFF");
pub const PANEL_STROKE: Color32 = rgb_from_hex!(b"#D0D4D9");
pub const WINDOW_FILL: Color32 = PANEL_BACKGROUND;

pub const BUTTON_FILL: Color32 = rgb_from_hex!(b"#FFFFFF");
pub const TEXT_COLOR: Color32 = rgb_from_hex!(b"#1f2125");

// --- Sizes

pub const SUBSCRIPT_FONT_SIZE: f32 = 14.0;
pub const ICON_FONT_SIZE: f32 = 24.0;
pub const BUTTON_SIZE: f32 = 60.0;
pub const NODE_SIZE: f32 = 60.0;
pub const BUTTON_HORIZONTAL_SPACING: f32 = 25.0;
pub const BUTTON_VERTICAL_SPACING: f32 = 25.0;
pub const BUTTON_FROM_BORDER_PADDING: f32 = 15.0;
pub const NODE_STROKE_WIDTH: f32 = 1.5;
pub const BUTTON_STROKE_WIDTH: f32 = NODE_STROKE_WIDTH;
pub const BUTTON_CORNER_RADIUS: f32 = 4.0;

// --- Widget theme

pub fn dark_theme_widgets() -> Widgets {
    Widgets {
        noninteractive: light_widget_noninteractive(),
        inactive: light_widget_inactive(),
        hovered: light_widget_hovered(),
        active: light_widget_active(),
        open: light_widget_open(),
    }
}

fn light_widget_noninteractive() -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: PANEL_BACKGROUND,
        weak_bg_fill: BUTTON_FILL,
        bg_stroke: Stroke::new(BUTTON_STROKE_WIDTH, PANEL_STROKE),
        fg_stroke: Stroke::new(BUTTON_STROKE_WIDTH, Color32::from_gray(60)),
        corner_radius: BUTTON_CORNER_RADIUS.into(),
        expansion: 0.0,
    }
}

fn light_widget_inactive() -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: BUTTON_FILL,
        weak_bg_fill: BUTTON_FILL,
        bg_stroke: Stroke::new(BUTTON_STROKE_WIDTH, PANEL_STROKE),
        fg_stroke: Stroke::NONE,
        corner_radius: BUTTON_CORNER_RADIUS.into(),
        expansion: 0.0,
    }
}

fn light_widget_hovered() -> WidgetVisuals {
    let hovered_fill = Color32::from_rgb(
        (BUTTON_FILL.r() as u16).saturating_sub(20) as u8,
        (BUTTON_FILL.g() as u16).saturating_sub(20) as u8,
        (BUTTON_FILL.b() as u16).saturating_sub(20) as u8,
    );
    WidgetVisuals {
        bg_fill: hovered_fill,
        weak_bg_fill: BUTTON_FILL,
        bg_stroke: Stroke::new(BUTTON_STROKE_WIDTH, SELECTED_NODE_STROKE),
        fg_stroke: Stroke::new(BUTTON_STROKE_WIDTH, TEXT_COLOR),
        corner_radius: BUTTON_CORNER_RADIUS.into(),
        expansion: 1.0,
    }
}

fn light_widget_active() -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: SELECTED_NODE_FILL,
        weak_bg_fill: SELECTED_NODE_FILL,
        bg_stroke: Stroke::new(BUTTON_STROKE_WIDTH, SELECTED_NODE_STROKE),
        fg_stroke: Stroke::new(BUTTON_STROKE_WIDTH, Color32::BLACK),
        corner_radius: BUTTON_CORNER_RADIUS.into(),
        expansion: 1.0,
    }
}

fn light_widget_open() -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: BUTTON_FILL,
        weak_bg_fill: BUTTON_FILL,
        bg_stroke: Stroke::new(BUTTON_STROKE_WIDTH, WORKING_NODE_STROKE),
        fg_stroke: Stroke::new(BUTTON_STROKE_WIDTH, TEXT_COLOR),
        corner_radius: BUTTON_CORNER_RADIUS.into(),
        expansion: 0.0,
    }
}

pub fn setup_visuals() -> Visuals {
    let mut visuals = egui::Visuals::light();
    visuals.dark_mode = true;
    visuals.selection = egui::style::Selection {
        bg_fill: SELECTED_NODE_FILL,
        stroke: egui::Stroke {
                width: 2.0,
            color: SELECTED_NODE_STROKE,
        },
    };

    visuals.extreme_bg_color = egui::Color32::WHITE;
    visuals.window_fill = WINDOW_FILL;
    visuals.panel_fill = PANEL_BACKGROUND;
    visuals.override_text_color = Some(TEXT_COLOR);
    visuals.widgets = dark_theme_widgets();
    visuals.window_shadow = egui::Shadow::NONE;
    visuals.window_corner_radius = BUTTON_CORNER_RADIUS.into();
    visuals
}
