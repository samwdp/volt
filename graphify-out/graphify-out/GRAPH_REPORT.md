# Graph Report - graphify-out  (2026-07-29)

## Corpus Check
- 4 files · ~1,001,296 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 224 nodes · 223 edges · 2 communities
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `86ecb4a5`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Communities (300 total, 26 thin omitted)
- Graph Report - volt  (2026-07-29)

## God Nodes (most connected - your core abstractions)
1. `Communities (300 total, 26 thin omitted)` - 213 edges
2. `Graph Report - volt  (2026-07-29)` - 11 edges
3. `Corpus Check` - 1 edges
4. `Summary` - 1 edges
5. `Graph Freshness` - 1 edges
6. `Community Hubs (Navigation)` - 1 edges
7. `God Nodes (most connected - your core abstractions)` - 1 edges
8. `Surprising Connections (you probably didn't know these)` - 1 edges
9. `Import Cycles` - 1 edges
10. `Community 0 - "String"` - 1 edges

## Surprising Connections (you probably didn't know these)
- None detected - all connections are within the same source files.

## Communities (2 total, 0 thin omitted)

### Community 0 - "Communities (300 total, 26 thin omitted)"
Cohesion: 0.01
Nodes (213): Communities (300 total, 26 thin omitted), Community 0 - "String", Community 100 - "shell/browser.rs", Community 101 - "RString", Community 102 - "LspInlineCompletionItem", Community 103 - "common.rs", Community 104 - "Option", Community 105 - "editor-lsp/src/lib.rs" (+205 more)

### Community 1 - "Graph Report - volt  (2026-07-29)"
Cohesion: 0.18
Nodes (10): Community Hubs (Navigation), Corpus Check, God Nodes (most connected - your core abstractions), Graph Freshness, Graph Report - volt  (2026-07-29), Import Cycles, Knowledge Gaps, Suggested Questions (+2 more)

## Knowledge Gaps
- **221 isolated node(s):** `Corpus Check`, `Summary`, `Graph Freshness`, `Community Hubs (Navigation)`, `God Nodes (most connected - your core abstractions)` (+216 more)
  These have ≤1 connection - possible missing edges or undocumented components.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Communities (300 total, 26 thin omitted)` connect `Communities (300 total, 26 thin omitted)` to `Graph Report - volt  (2026-07-29)`?**
  _High betweenness centrality (0.998) - this node is a cross-community bridge._
- **Why does `Graph Report - volt  (2026-07-29)` connect `Graph Report - volt  (2026-07-29)` to `Communities (300 total, 26 thin omitted)`?**
  _High betweenness centrality (0.088) - this node is a cross-community bridge._
- **What connects `Corpus Check`, `Summary`, `Graph Freshness` to the rest of the system?**
  _221 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Communities (300 total, 26 thin omitted)` be split into smaller, more focused modules?**
  _Cohesion score 0.009389671361502348 - nodes in this community are weakly interconnected._