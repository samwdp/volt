use std::path::Path;

use editor_buffer::TextPoint;

use super::types::*;

impl LspClientManager {
    pub fn inline_completion(
        &self,
        path: &Path,
        position: TextPoint,
        options: LspFormattingOptions,
    ) -> Result<Option<LspInlineCompletionItem>, LspClientError> {
        let (version, sessions) = self.tracked_sessions_and_version_for_path(path)?;
        for session in sessions {
            if !is_copilot_server(session.server_id()) {
                continue;
            }
            session.did_focus(path)?;
            if let Some(item) = session.inline_completion(path, version, position, options)? {
                return Ok(Some(item));
            }
        }
        Ok(None)
    }
}
