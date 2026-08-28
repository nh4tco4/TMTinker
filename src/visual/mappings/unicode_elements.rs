#![allow(dead_code)]
/* === TOOLS === */
pub const TOOL_MOVE: &str = "✋";
pub const TOOL_DELETE: &str = "❌";
pub const TOOL_LEFT: &str = "l";
pub const TOOL_RIGHT: &str = "r";

pub const TOOL_LEFTWORD: &str = "L";
pub const TOOL_RIGHTWORD: &str = "R";
pub const TOOL_LINK: &str = "↔";
// pub const TOOL_PLACE_SIGN: &str = "Λ";
pub const TOOL_PLACE_SIGN: &str = "λ";
pub const TOOL_START: &str = "○";
pub const TOOL_END: &str = "◎";
pub const TOOL_COPY: &str = "K";
pub const TOOL_SUBMACHINE: &str = "@";
pub const TOOL_INVERSE_WORD: &str = "⟲";
pub const TOOL_SHIFT_ONE_WORD_BACKWARDS: &str = "Sb";
pub const TOOL_SKIP_SEQUENCE_LEFT: &str = "Pl";
pub const TOOL_SKIP_SEQUENCE_RIGHT: &str = "Pr";
pub const TOOL_DELETE_WORD_LEFT: &str = "Dl";
pub const TOOL_DELETE_WORD_RIGHT: &str = "Dr";
pub const TOOL_COPY_NTH_WORD: &str = "Cn";

/* === NODES === */
pub const NODE_LEFT: &str = TOOL_LEFT;
pub const NODE_RIGHT: &str = TOOL_RIGHT;
pub const NODE_WORD_LEFT: &str = TOOL_LEFTWORD;
pub const NODE_WORD_RIGHT: &str = TOOL_RIGHTWORD;
pub const NODE_SPACE: &str = TOOL_PLACE_SIGN;
pub const NODE_START: &str = TOOL_START;
pub const NODE_END: &str = TOOL_END;
pub const NODE_COPY: &str = TOOL_COPY;
pub const NODE_SUBMACHINE: &str = TOOL_SUBMACHINE;
pub const NODE_INVERSE_WORD: &str = TOOL_INVERSE_WORD;
pub const NODE_SHIFT_ONE_WORD_BACKWARDS: &str = TOOL_SHIFT_ONE_WORD_BACKWARDS;
pub const NODE_SKIP_SEQUENCE_LEFT: &str = TOOL_SKIP_SEQUENCE_LEFT;
pub const NODE_SKIP_SEQUENCE_RIGHT: &str = TOOL_SKIP_SEQUENCE_RIGHT;
pub const NODE_DELETE_WORD_LEFT: &str = TOOL_DELETE_WORD_LEFT;
pub const NODE_DELETE_WORD_RIGHT: &str = TOOL_DELETE_WORD_RIGHT;
pub const NODE_COPY_NTH_WORD: &str = TOOL_COPY_NTH_WORD;

/* === Controls === */
pub const CONTROL_START: &str = "▶";
pub const CONTROL_PAUSE: &str = "⏸";
pub const CONTROL_STEP: &str = "⏵";
pub const CONTROL_RUN: &str = "⏩";
pub const CONTROL_SKIP: &str = "⏭";
pub const CONTROL_RESET: &str = "↺";
pub const CONTROL_STOP: &str = "■";

pub const OPTIONS_RUN: &str = CONTROL_START;
pub const OPTIONS_VALIDATE: &str = "🔧";
pub const OPTIONS_SAVE: &str = "💾";
pub const OPTIONS_EXPORT: &str = "📤";
pub const OPTIONS_MENU: &str = "☰";
pub const OPTIONS_HELP: &str = "?";

/* === Miscellanious === */
pub const UNKNOWN: &str = "?";

pub const SPACE: char = ' ';
pub const LAMBDA: char = 'λ';
pub const NOT_EQUAL: char = '≠';
