from pathlib import Path

p = Path(r"P:\volt\user\sdk\src\lib.rs")
text = p.read_text(encoding="utf-8")
old = (
    "pub mod dap_hooks {\n"
    '    pub const START: &str = "dap.session-start";\n'
    '    pub const STOP: &str = "dap.session-stop";'
)
new = (
    "pub mod dap_hooks {\n"
    '    pub const START: &str = "dap.session-start";\n'
    '    pub const START_LAST: &str = "dap.session-start-last";\n'
    '    pub const START_RECENT: &str = "dap.session-start-recent";\n'
    '    pub const STOP: &str = "dap.session-stop";'
)
if old not in text:
    raise SystemExit("old block not found")
p.write_text(text.replace(old, new, 1), encoding="utf-8")
print("ok")
