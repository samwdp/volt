use super::*;

#[derive(Debug, Clone)]
struct CommandLineCompletionState {
    seed: String,
    matches: Vec<String>,
    index: usize,
}

#[derive(Debug, Clone)]
pub(super) struct CommandLineOverlay {
    input: InputField,
    completion: Option<CommandLineCompletionState>,
}

impl CommandLineOverlay {
    pub(super) fn new() -> Self {
        let mut input = InputField::new(":");
        input.set_placeholder(Some(
            "command, !shell command, or %s/find/replace/g".to_owned(),
        ));
        Self {
            input,
            completion: None,
        }
    }

    pub(super) fn input(&self) -> &InputField {
        &self.input
    }

    pub(super) fn text(&self) -> &str {
        self.input.text()
    }

    pub(super) fn append_text(&mut self, text: &str) {
        let filtered: String = text
            .chars()
            .filter(|character| !matches!(character, '\r' | '\n'))
            .collect();
        if filtered.is_empty() {
            return;
        }
        self.input.insert_text(&filtered);
        self.completion = None;
    }

    pub(super) fn backspace(&mut self) {
        self.input.backspace();
        self.completion = None;
    }

    pub(super) fn delete_forward(&mut self) {
        self.input.delete_forward();
        self.completion = None;
    }

    pub(super) fn move_left(&mut self) {
        let _ = self.input.move_left();
        self.completion = None;
    }

    pub(super) fn move_right(&mut self) {
        let _ = self.input.move_right();
        self.completion = None;
    }

    pub(super) fn cycle_completion(&mut self, matches: Vec<String>, reverse: bool) {
        if matches.is_empty() {
            self.completion = None;
            return;
        }
        let seed = self.input.text().to_owned();
        let same_cycle = self
            .completion
            .as_ref()
            .is_some_and(|state| state.matches == matches && self.input.text() != state.seed);
        let index = if let Some(state) = &self.completion {
            if same_cycle {
                let len = state.matches.len();
                if reverse {
                    state.index.checked_sub(1).unwrap_or(len.saturating_sub(1))
                } else {
                    (state.index + 1) % len
                }
            } else if reverse {
                matches.len().saturating_sub(1)
            } else {
                0
            }
        } else if reverse {
            matches.len().saturating_sub(1)
        } else {
            0
        };
        if let Some(selected) = matches.get(index) {
            self.input.set_text(selected);
            self.completion = Some(CommandLineCompletionState {
                seed,
                matches,
                index,
            });
        }
    }
}
