//! Picker Extra Keybind dispatch seam.
//!
//! Pure resolve of instance extras against a chord. Shared Popup navigation stays
//! outside this module (fallthrough). Callers snapshot context, close the picker,
//! then run the bound command.

/// Chord → command declaration copied onto one open picker instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerExtraKeybind {
    chord: String,
    command_name: String,
}

impl PickerExtraKeybind {
    /// Creates one extra keybind.
    pub fn new(chord: impl Into<String>, command_name: impl Into<String>) -> Self {
        Self {
            chord: chord.into(),
            command_name: command_name.into(),
        }
    }

    /// Returns the chord string (for example `Ctrl+d`).
    pub fn chord(&self) -> &str {
        &self.chord
    }

    /// Returns the bound command name.
    pub fn command_name(&self) -> &str {
        &self.command_name
    }
}

/// Selected picker row snapshot for one-shot command context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerSelectedRow {
    id: String,
    label: String,
    path: Option<String>,
}

impl PickerSelectedRow {
    /// Creates a selected-row snapshot.
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        path: Option<impl Into<String>>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            path: path.map(Into::into),
        }
    }

    /// Returns the row id.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the row label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns an optional path associated with the row.
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }
}

/// Exportable QuickFix-shaped row carried in one-shot context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickerExportableRow {
    id: String,
    path: String,
    line: usize,
    column: usize,
    label: String,
}

impl PickerExportableRow {
    /// Creates one exportable QuickFix row snapshot.
    pub fn new(
        id: impl Into<String>,
        path: impl Into<String>,
        line: usize,
        column: usize,
        label: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            path: path.into(),
            line,
            column,
            label: label.into(),
        }
    }

    /// Returns the row id.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the file path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the 0-based line.
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Returns the 0-based column.
    pub const fn column(&self) -> usize {
        self.column
    }

    /// Returns the display label.
    pub fn label(&self) -> &str {
        &self.label
    }
}

/// One-shot picker context snapshotted when an extra fires.
///
/// Consumed once by the bound command; not a sticky last-selection service.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PickerOneShotContext {
    selected: Option<PickerSelectedRow>,
    exportable_quickfix: Vec<PickerExportableRow>,
}

impl PickerOneShotContext {
    /// Builds a context from live picker selection and optional Quickfix rows.
    pub fn new(
        selected: Option<PickerSelectedRow>,
        exportable_quickfix: Vec<PickerExportableRow>,
    ) -> Self {
        Self {
            selected,
            exportable_quickfix,
        }
    }

    /// Returns the selected row, when any.
    pub fn selected(&self) -> Option<&PickerSelectedRow> {
        self.selected.as_ref()
    }

    /// Returns exportable Quickfix rows present on the instance.
    pub fn exportable_quickfix(&self) -> &[PickerExportableRow] {
        &self.exportable_quickfix
    }
}

/// Outcome of resolving a chord against picker-instance extras.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerExtraDispatch {
    /// Chord is not an instance extra — shared Popup / Global may handle it.
    Fallthrough,
    /// Extra matched: close picker and run command with snapshotted context.
    Fire {
        /// Bound command name.
        command_name: String,
        /// One-shot context for the command handler.
        context: PickerOneShotContext,
        /// Always true: extras close the picker before command execution.
        close_picker: bool,
    },
}

/// Resolves a chord against picker-instance extras.
///
/// Matching extras always request close and include a defined context (selected
/// may be `None`; create-row / empty selection still yield a context value).
pub fn resolve_picker_extra(
    extras: &[PickerExtraKeybind],
    chord: &str,
    selected: Option<PickerSelectedRow>,
    exportable_quickfix: Vec<PickerExportableRow>,
) -> PickerExtraDispatch {
    let Some(extra) = extras.iter().find(|extra| extra.chord() == chord) else {
        return PickerExtraDispatch::Fallthrough;
    };
    PickerExtraDispatch::Fire {
        command_name: extra.command_name().to_owned(),
        context: PickerOneShotContext::new(selected, exportable_quickfix),
        close_picker: true,
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
