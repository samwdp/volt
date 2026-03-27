use std::{collections::BTreeMap, time::Instant};

use editor_buffer::{TextPoint, TextRange};
use editor_core::BufferId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputMode {
    Normal,
    Insert,
    Replace,
    Visual,
}

impl InputMode {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Normal => "NORMAL",
            Self::Insert => "INSERT",
            Self::Replace => "REPLACE",
            Self::Visual => "VISUAL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VimOperator {
    Delete,
    Change,
    Yank,
    ToggleCase,
    Lowercase,
    Uppercase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VimFindKind {
    ForwardTo,
    BackwardTo,
    ForwardBefore,
    BackwardAfter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VimSearchDirection {
    Forward,
    Backward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShellMotion {
    Left,
    Down,
    Up,
    Right,
    WordForward,
    WordBackward,
    WordEnd,
    BigWordForward,
    BigWordBackward,
    BigWordEnd,
    SentenceForward,
    SentenceBackward,
    ParagraphForward,
    ParagraphBackward,
    WordEndBackward,
    BigWordEndBackward,
    MatchPair,
    LineStart,
    LineFirstNonBlank,
    LineEnd,
    ScreenTop,
    ScreenMiddle,
    ScreenBottom,
    FirstLine,
    LastLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScrollCommand {
    HalfPageDown,
    HalfPageUp,
    PageDown,
    PageUp,
    LineDown,
    LineUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VimTextObjectKind {
    Word,
    BigWord,
    Sentence,
    Paragraph,
    Delimited { open: char, close: char },
    Tag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VimPending {
    Operator {
        operator: VimOperator,
        count: usize,
    },
    Format {
        count: usize,
    },
    FindTarget {
        operator: Option<VimOperator>,
        kind: VimFindKind,
        count: usize,
    },
    GPrefix {
        operator: Option<VimOperator>,
        line_target: Option<usize>,
    },
    TextObject {
        operator: VimOperator,
        around: bool,
        count: usize,
    },
    VisualTextObject {
        around: bool,
        count: usize,
    },
    ReplaceChar {
        count: usize,
    },
    Register,
    MarkSet,
    MarkJump {
        linewise: bool,
    },
    MacroRecord,
    MacroPlayback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LastFind {
    pub(crate) kind: VimFindKind,
    pub(crate) target: char,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LastSearch {
    pub(crate) direction: VimSearchDirection,
    pub(crate) query: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum YankRegister {
    Character(String),
    Line(String),
    Block(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FormatterSpec {
    pub(crate) language_id: String,
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
}

impl FormatterSpec {
    pub(crate) fn from_hook_detail(detail: &str) -> Result<Self, String> {
        let mut parts = detail
            .split('|')
            .map(str::trim)
            .filter(|part| !part.is_empty());
        let language_id = parts
            .next()
            .ok_or_else(|| "formatter registration missing language id".to_owned())?;
        let program = parts
            .next()
            .ok_or_else(|| "formatter registration missing program".to_owned())?;
        let args = parts.map(|part| part.to_owned()).collect();
        Ok(Self {
            language_id: language_id.to_owned(),
            program: program.to_owned(),
            args,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct FormatterRegistry {
    formatters: BTreeMap<String, FormatterSpec>,
}

impl FormatterRegistry {
    pub(crate) fn register(&mut self, spec: FormatterSpec) -> Result<(), String> {
        if let Some(existing) = self.formatters.get(&spec.language_id) {
            if existing == &spec {
                return Ok(());
            }
            return Err(format!(
                "formatter already registered for language `{}`",
                spec.language_id
            ));
        }
        self.formatters.insert(spec.language_id.clone(), spec);
        Ok(())
    }

    pub(crate) fn formatter_for_language(&self, language_id: &str) -> Option<&FormatterSpec> {
        self.formatters.get(language_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VimMark {
    pub(crate) buffer_id: BufferId,
    pub(crate) point: TextPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VimVisualSnapshot {
    pub(crate) buffer_id: BufferId,
    pub(crate) anchor: TextPoint,
    pub(crate) head: TextPoint,
    pub(crate) kind: VisualSelectionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum VimRecordedInput {
    Text(String),
    Chord(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum VisualSelectionKind {
    #[default]
    Character,
    Line,
    Block,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BlockSelection {
    pub(crate) start_line: usize,
    pub(crate) end_line: usize,
    pub(crate) start_col: usize,
    pub(crate) end_col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VisualSelection {
    Range(TextRange),
    Block(BlockSelection),
}

#[derive(Debug, Clone)]
pub(crate) struct BlockInsertState {
    pub(crate) selection: BlockSelection,
    pub(crate) insert_col: usize,
    pub(crate) origin_line: usize,
    pub(crate) original_line: String,
}

#[derive(Debug, Clone)]
pub(crate) struct VimBufferState {
    pub(crate) input_mode: InputMode,
    pub(crate) count: Option<usize>,
    pub(crate) pending: Option<VimPending>,
    pub(crate) visual_anchor: Option<TextPoint>,
    pub(crate) visual_kind: VisualSelectionKind,
    pub(crate) active_register: Option<char>,
    pub(crate) pending_change_prefix: Option<VimRecordedInput>,
    pub(crate) recording_change: bool,
    pub(crate) finish_change_on_normal: bool,
    pub(crate) finish_change_after_input: bool,
    pub(crate) change_buffer: Vec<VimRecordedInput>,
    pub(crate) block_insert: Option<BlockInsertState>,
}

impl Default for VimBufferState {
    fn default() -> Self {
        Self {
            input_mode: InputMode::Normal,
            count: None,
            pending: None,
            visual_anchor: None,
            visual_kind: VisualSelectionKind::Character,
            active_register: None,
            pending_change_prefix: None,
            recording_change: false,
            finish_change_on_normal: false,
            finish_change_after_input: false,
            change_buffer: Vec::new(),
            block_insert: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct YankFlash {
    pub(crate) buffer_id: BufferId,
    pub(crate) selection: VisualSelection,
    pub(crate) until: Instant,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct VimState {
    pub(crate) count: Option<usize>,
    pub(crate) pending: Option<VimPending>,
    pub(crate) visual_anchor: Option<TextPoint>,
    pub(crate) visual_kind: VisualSelectionKind,
    pub(crate) last_find: Option<LastFind>,
    pub(crate) last_search: Option<LastSearch>,
    pub(crate) yank: Option<YankRegister>,
    pub(crate) registers: BTreeMap<char, YankRegister>,
    pub(crate) active_register: Option<char>,
    pub(crate) marks: BTreeMap<char, VimMark>,
    pub(crate) last_visual: Option<VimVisualSnapshot>,
    pub(crate) pending_change_prefix: Option<VimRecordedInput>,
    pub(crate) recording_macro: Option<char>,
    pub(crate) macro_buffer: Vec<VimRecordedInput>,
    pub(crate) macros: BTreeMap<char, Vec<VimRecordedInput>>,
    pub(crate) last_macro: Option<char>,
    pub(crate) skip_next_macro_input: bool,
    pub(crate) recording_change: bool,
    pub(crate) finish_change_on_normal: bool,
    pub(crate) finish_change_after_input: bool,
    pub(crate) change_buffer: Vec<VimRecordedInput>,
    pub(crate) last_change: Vec<VimRecordedInput>,
    pub(crate) replaying: bool,
    pub(crate) block_insert: Option<BlockInsertState>,
}

impl VimState {
    pub(crate) fn push_count_digit(&mut self, digit: usize) {
        self.count = Some(
            self.count
                .unwrap_or(0)
                .saturating_mul(10)
                .saturating_add(digit),
        );
    }

    pub(crate) fn take_count(&mut self) -> Option<usize> {
        self.count.take()
    }

    pub(crate) fn take_count_or_one(&mut self) -> usize {
        self.take_count().unwrap_or(1)
    }

    pub(crate) fn clear_transient(&mut self) {
        self.count = None;
        self.pending = None;
        self.pending_change_prefix = None;
    }

    pub(crate) fn active_buffer_state(&self, input_mode: InputMode) -> VimBufferState {
        VimBufferState {
            input_mode,
            count: self.count,
            pending: self.pending,
            visual_anchor: self.visual_anchor,
            visual_kind: self.visual_kind,
            active_register: self.active_register,
            pending_change_prefix: self.pending_change_prefix.clone(),
            recording_change: self.recording_change,
            finish_change_on_normal: self.finish_change_on_normal,
            finish_change_after_input: self.finish_change_after_input,
            change_buffer: self.change_buffer.clone(),
            block_insert: self.block_insert.clone(),
        }
    }

    pub(crate) fn apply_active_buffer_state(
        &mut self,
        input_mode: &mut InputMode,
        state: &VimBufferState,
    ) {
        *input_mode = state.input_mode;
        self.count = state.count;
        self.pending = state.pending;
        self.visual_anchor = state.visual_anchor;
        self.visual_kind = state.visual_kind;
        self.active_register = state.active_register;
        self.pending_change_prefix
            .clone_from(&state.pending_change_prefix);
        self.recording_change = state.recording_change;
        self.finish_change_on_normal = state.finish_change_on_normal;
        self.finish_change_after_input = state.finish_change_after_input;
        self.change_buffer.clone_from(&state.change_buffer);
        self.block_insert.clone_from(&state.block_insert);
    }
}
