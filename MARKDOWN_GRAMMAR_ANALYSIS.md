# Tree-Sitter Markdown Grammar Analysis for Volt

## Current Architecture

### File Paths & Registration

**Markdown Language Configuration**
- File: P:\volt/user/lang/markdown.rs
- Exports two separate language configurations:
  1. `syntax_language()` - Markdown **block** grammar
  2. `inline_syntax_language()` - Markdown **inline** grammar

**Registration Point**
- File: P:\volt/user/lang/mod.rs (lines 9-14)
- Exports `syntax_languages()` function that returns both:
  ```rust
  pub fn syntax_languages() -> Vec<LanguageConfiguration> {
      vec![
          rust::syntax_language(),
          markdown::syntax_language(),      // block grammar
          markdown::inline_syntax_language(), // inline grammar
      ]
  }
  ```

**Bootstrap Registration**
- File: P:\volt/crates/volt/src/main.rs (line ~335)
- Both are registered via: `syntax_registry.register_all(user::syntax_languages())?;`

### How Grammar Selection Works Currently

**SyntaxRegistry Structure**
- File: P:\volt/crates/editor-syntax/src/lib.rs (lines 473-478)
- Contains:
  - `languages: BTreeMap<String, LanguageConfiguration>` - Maps language_id to config
  - `extensions: BTreeMap<String, String>` - Maps file extension to language_id
  - `loaded: BTreeMap<String, LoadedLanguage>` - Cached parsed grammars

**Selection Mechanism**
- File: P:\volt/crates/editor-syntax/src/lib.rs (lines 672-703)
- `highlight_buffer_for_path()` and `highlight_buffer_for_extension()` methods:
  1. Look up file extension in `extensions` map
  2. Find corresponding language_id (only **one** per extension)
  3. Load that one grammar
  4. Parse entire buffer with single parser

**Current Problem**: The registry uses `extension -> language_id (1:1)` mapping. Only the **block** grammar is automatically used for .md files.

## Tree-Sitter Markdown Grammar Architecture

### Design Rationale
The `tree-sitter-markdown` repository provides **two separate grammars**:
- `tree-sitter-markdown` (block): Handles overall document structure (headings, lists, code blocks, etc.)
- `tree-sitter-markdown-inline` (inline): Handles inline formatting (emphasis, links, code spans, etc.)

### How They're Meant to Work Together
According to tree-sitter-markdown architecture, inline content should be parsed as **included ranges** within block-level elements. The block parser identifies regions (e.g., paragraph content), and the inline parser targets those ranges specifically.

## Current Registration Details

### Block Grammar (P:\volt/user/lang/markdown.rs, lines 42-63)
```rust
pub fn syntax_language() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "markdown",                    // language_id
        ["md", "markdown"],            // file extensions
        GrammarSource::new(
            "https://github.com/tree-sitter-grammars/tree-sitter-markdown.git",
            "tree-sitter-markdown",    // grammar directory
            "src",                     // source subdirectory
            "tree-sitter-markdown",    // install directory name
            "tree_sitter_markdown",    // exported C function symbol
        ),
        [...capture mappings...]
    )
}
```

### Inline Grammar (P:\volt/user/lang/markdown.rs, lines 66-87)
```rust
pub fn inline_syntax_language() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "markdown-inline",             // language_id
        [] as [&str; 0],               // NO FILE EXTENSIONS!
        GrammarSource::new(
            "https://github.com/tree-sitter-grammars/tree-sitter-markdown.git",
            "tree-sitter-markdown-inline",
            "src",
            "tree-sitter-markdown-inline",
            "tree_sitter_markdown_inline",
        ),
        [...capture mappings...]
    )
}
```

## Parsing Flow

### Current Single-Parser Flow
File opens (.md) 
  → refresh_buffer_syntax() called [P:\volt/crates/editor-sdl/src/lib.rs]
  → registry.highlight_buffer_for_path(&path, &text)
  → Extension ".md" → language_id "markdown" (block only)
  → load_language("markdown") into LoadedLanguage {
      language: tree_sitter_markdown,
      query: highlights.scm from block grammar,
      capture_mappings: block mappings
    }
  → highlight_loaded_language():
      let mut parser = Parser::new()           [line 870]
      parser.set_language(&loaded.language)   [line 872]
      tree = parser.parse(&source, None)      [line 879-880]
      cursor.matches(&query, tree.root_node(), source)
      Returns SyntaxSnapshot with only block-level highlights

## Recommended Changes to Support Both Grammars

### Approach 1: Dual-Parser with Included Ranges (RECOMMENDED)

**Benefits**: 
- Follows tree-sitter-markdown design intent
- Inline details highlight within block contexts
- Single SyntaxSnapshot output (seamless)

**Key Functions/Files to Modify**:

1. P:\volt/crates/editor-syntax/src/lib.rs, line 865:
   - Modify `highlight_loaded_language()` to accept optional secondary grammar
   - Add call to `parser.set_included_ranges()` for inline grammar
   - Merge both parser outputs before returning SyntaxSnapshot

2. P:\volt/crates/editor-syntax/src/lib.rs, struct SyntaxRegistry (line 473):
   - Add field: `dual_grammars: BTreeMap<String, String>` 
   - Maps primary language_id → secondary language_id

3. P:\volt/crates/editor-syntax/src/lib.rs, impl SyntaxRegistry:
   - Add method: `register_dual_grammar(primary, secondary) -> Result<(), SyntaxError>`
   - Modify `highlight_buffer_for_extension()` to check dual_grammars mapping

4. P:\volt/user/lang/mod.rs, line 9:
   - Add new function: `syntax_dual_grammars() -> Vec<(&'static str, &'static str)>`
   - Return vec![("markdown", "markdown-inline")]

5. P:\volt/crates/volt/src/main.rs, line ~335:
   - After `syntax_registry.register_all(user::syntax_languages())?;`
   - Add: Register dual grammar relationships from user lib

### Key tree-sitter API Functions Available

From tree-sitter Rust bindings (crate `tree-sitter`):
```rust
impl Parser {
    pub fn set_language(&mut self, language: &Language) -> Result<()>;
    pub fn set_included_ranges(&mut self, ranges: &[Range]) -> Result<()>;
    pub fn parse(&mut self, source: &str, old_tree: Option<&Tree>) -> Option<Tree>;
}

pub struct Range {
    pub start_byte: u32,
    pub end_byte: u32,
    pub start_point: Point,
    pub end_point: Point,
}
```

### Approach 2: Sequential Parsing (SIMPLER, LESS CORRECT)

Just parse buffer twice and merge highlights - doesn't use included_ranges, so inline highlights won't respect block boundaries properly. Not recommended.

## Error Handling

Add to SyntaxError enum (P:\volt/crates/editor-syntax/src/lib.rs, line ~334):
```rust
pub enum SyntaxError {
    // ... existing variants ...
    UnknownSecondaryLanguage { primary: String, secondary: String },
    IncludedRangesFailed { language_id: String, message: String },
}
```

## Summary Table

| Aspect | Current | Recommended |
|--------|---------|-------------|
| Grammars Used | Block only | Block + Inline |
| Parser Count | 1 per buffer | 2 per buffer (with ranges) |
| Selection Method | Extension map | Extension → dual grammar lookup |
| Highlight Sources | Block captures | Block + Inline captures |
| Architecture File | P:\volt/crates/editor-syntax/src/lib.rs | Same |
| Config File | P:\volt/user/lang/markdown.rs | Same |
| Bootstrap File | P:\volt/crates/volt/src/main.rs | Same (+ dual_grammars) |
| Registration File | P:\volt/user/lang/mod.rs | Same (+ new function) |

## Implementation Checklist

- [ ] Add `dual_grammars` field to SyntaxRegistry struct
- [ ] Implement `register_dual_grammar()` method
- [ ] Modify `highlight_loaded_language()` signature to accept Option<&LoadedLanguage>
- [ ] Implement included_ranges extraction logic from block tree
- [ ] Call parser.set_included_ranges() in dual-parser path
- [ ] Update `highlight_buffer_for_extension()` to lookup dual grammar
- [ ] Add `syntax_dual_grammars()` export in user/lang/mod.rs
- [ ] Wire dual grammar registration in main.rs bootstrap
- [ ] Add tests for dual-grammar registration and highlighting
- [ ] Verify highlights from both block and inline grammars appear in output
