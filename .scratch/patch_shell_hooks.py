from pathlib import Path

p = Path(r"P:\volt\crates\editor-sdl\src\shell\mod.rs")
text = p.read_text(encoding="utf-8")

old = (
    "const HOOK_DAP_START: &str = dap_hooks::START;\n"
    "const HOOK_DAP_STOP: &str = dap_hooks::STOP;"
)
new = (
    "const HOOK_DAP_START: &str = dap_hooks::START;\n"
    "const HOOK_DAP_START_LAST: &str = dap_hooks::START_LAST;\n"
    "const HOOK_DAP_START_RECENT: &str = dap_hooks::START_RECENT;\n"
    "const HOOK_DAP_STOP: &str = dap_hooks::STOP;"
)
if old not in text:
    raise SystemExit("hook consts not found")
text = text.replace(old, new, 1)
p.write_text(text, encoding="utf-8")
print("hooks ok")
