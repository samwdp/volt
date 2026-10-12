import json
from graphify.extract import collect_files, extract
from pathlib import Path

code_files = []
detect = json.loads(Path("graphify-out/.graphify_detect.json").read_text(encoding="utf-8-sig"))
for f in detect.get("files", {}).get("code", []):
    p = Path(f)
    code_files.extend(collect_files(p) if p.is_dir() else [p])

print(f"code files: {len(code_files)}", flush=True)
result = extract(code_files, cache_root=Path(r"P:\volt"))
Path("graphify-out/.graphify_ast.json").write_text(
    json.dumps(result, indent=2, ensure_ascii=False), encoding="utf-8"
)
nodes = result["nodes"]
edges = result["edges"]
print(f"AST: {len(nodes)} nodes, {len(edges)} edges")
