from pathlib import Path
import re

text = Path(r"P:\volt\typing-profile.log").read_text(encoding="utf-8", errors="replace")
pat = re.compile(
    r"\[([\d.]+)\] frame=(\d+).*?events=(\d+).*?keydowns=(\d+).*?text_inputs=(\d+).*?"
    r'preview="([^"]*)".*?layout_sync=([\d.]+)ms.*?lsp=([\d.]+)ms.*?git=([\d.]+)ms.*?'
    r"acp=([\d.]+)ms.*?render=([\d.]+)ms.*?present=([\d.]+)ms.*?total=([\d.]+)ms"
)
seen = set()
rows = []
for m in pat.finditer(text):
    g = m.groups()
    if g[1] in seen:
        continue
    seen.add(g[1])
    rows.append(
        {
            "ts": float(g[0]),
            "frame": int(g[1]),
            "events": int(g[2]),
            "kd": int(g[3]),
            "ti": int(g[4]),
            "preview": g[5],
            "layout": float(g[6]),
            "lsp": float(g[7]),
            "git": float(g[8]),
            "acp": float(g[9]),
            "render": float(g[10]),
            "present": float(g[11]),
            "total": float(g[12]),
        }
    )

print("unique frames", len(rows))
print("\n=== j keydown frames ===")
js = [r for r in rows if r["kd"] and r["preview"] == "j"]
for r in js[:20]:
    print(
        f"frame={r['frame']} total={r['total']:.1f} layout={r['layout']:.1f} "
        f"render={r['render']:.1f} present={r['present']:.1f} lsp={r['lsp']:.1f} git={r['git']:.1f}"
    )
print(f"j count={len(js)} layout avg={sum(r['layout'] for r in js)/max(len(js),1):.1f}")

print("\n=== git>200ms frames and next keydown ===")
for i, r in enumerate(rows):
    if r["git"] < 200:
        continue
    print(
        f"SPIKE frame={r['frame']} kd={r['kd']} git={r['git']:.0f} total={r['total']:.0f}"
    )
    for n in rows[i + 1 : i + 8]:
        if n["kd"] or n["ti"]:
            gap = n["ts"] - r["ts"]
            print(
                f"  next input +{gap*1000:.0f}ms frame={n['frame']} preview={n['preview']!r} "
                f"total={n['total']:.1f} layout={n['layout']:.1f}"
            )
            break
