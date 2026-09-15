import re
from collections import defaultdict
from pathlib import Path

text = Path(r"P:\volt\typing-profile.log").read_text(encoding="utf-8", errors="replace")
for line in text.splitlines()[:40]:
    print(line)

pat = re.compile(
    r"frame=(\d+).*?events=(\d+).*?keydowns=(\d+).*?text_inputs=(\d+).*?"
    r'preview="([^"]*)".*?handle=([\d.]+)ms.*?keydown_handle=([\d.]+)ms.*?'
    r"text_handle=([\d.]+)ms.*?text_inner=([\d.]+)ms.*?layout_sync=([\d.]+)ms.*?"
    r"picker_search=([\d.]+)ms.*?lsp=([\d.]+)ms.*?notifications=([\d.]+)ms.*?"
    r"autocomplete=([\d.]+)ms.*?hover=([\d.]+)ms.*?terminal=([\d.]+)ms.*?"
    r"syntax_apply=([\d.]+)ms.*?syntax_worker=([\d.]+)ms.*?git=([\d.]+)ms.*?"
    r"acp=([\d.]+)ms.*?render=([\d.]+)ms.*?present=([\d.]+)ms.*?total=([\d.]+)ms"
)

fields = [
    "frame",
    "events",
    "keydowns",
    "text_inputs",
    "preview",
    "handle",
    "keydown_handle",
    "text_handle",
    "text_inner",
    "layout_sync",
    "picker_search",
    "lsp",
    "notifications",
    "autocomplete",
    "hover",
    "terminal",
    "syntax_apply",
    "syntax_worker",
    "git",
    "acp",
    "render",
    "present",
    "total",
]
rows = []
for m in pat.finditer(text):
    d = {k: m.group(i + 1) for i, k in enumerate(fields)}
    for k in fields:
        if k == "preview":
            continue
        if k in ("frame", "events", "keydowns", "text_inputs"):
            d[k] = int(d[k])
        else:
            d[k] = float(d[k])
    rows.append(d)
print(f"parsed={len(rows)}")

comp_keys = [
    "keydown_handle",
    "text_handle",
    "layout_sync",
    "lsp",
    "autocomplete",
    "hover",
    "syntax_apply",
    "git",
    "acp",
    "render",
    "present",
    "handle",
]

print("\n=== slowest 25 frames ===")
for r in sorted(rows, key=lambda r: r["total"], reverse=True)[:25]:
    comps = {k: r[k] for k in comp_keys}
    top = sorted(comps.items(), key=lambda x: x[1], reverse=True)[:3]
    print(
        f"frame={r['frame']} total={r['total']:.1f} kd={r['keydowns']} "
        f"ti={r['text_inputs']} preview={r['preview']!r} top={top}"
    )

kd = [r for r in rows if r["keydowns"] > 0]
print(f"\nkeydown frames={len(kd)}")
print("\n=== slowest 30 keydown frames ===")
for r in sorted(kd, key=lambda r: r["total"], reverse=True)[:30]:
    comps = {k: r[k] for k in comp_keys}
    top = sorted(comps.items(), key=lambda x: x[1], reverse=True)[:4]
    print(
        f"frame={r['frame']} total={r['total']:.1f} kd={r['keydowns']} "
        f"preview={r['preview']!r} keydown_handle={r['keydown_handle']:.2f} top={top}"
    )

print("\n=== keydown frame component avg/p95/max ===")
for k in [
    "total",
    "keydown_handle",
    "handle",
    "layout_sync",
    "lsp",
    "git",
    "acp",
    "render",
    "present",
    "syntax_apply",
    "autocomplete",
]:
    vals = sorted(r[k] for r in kd)
    p95 = vals[int(0.95 * (len(vals) - 1))]
    print(f"{k}: avg={sum(vals)/len(vals):.2f} p95={p95:.2f} max={max(vals):.2f}")

pat2 = re.compile(
    r"\[([\d.]+)\] frame=(\d+).*?keydowns=(\d+).*?text_inputs=(\d+).*?"
    r'preview="([^"]*)".*?keydown_handle=([\d.]+)ms.*?lsp=([\d.]+)ms.*?'
    r"git=([\d.]+)ms.*?acp=([\d.]+)ms.*?render=([\d.]+)ms.*?total=([\d.]+)ms"
)
ev = []
for m in pat2.finditer(text):
    ts, frame, kdi, ti, prev, kh, lsp, git, acp, rend, tot = m.groups()
    if int(kdi) == 0 and int(ti) == 0:
        continue
    ev.append(
        (
            float(ts),
            int(frame),
            int(kdi),
            int(ti),
            prev,
            float(kh),
            float(lsp),
            float(git),
            float(acp),
            float(rend),
            float(tot),
        )
    )


def stats(name, xs):
    if not xs:
        print(name, "empty")
        return
    tots = [x[-1] for x in xs]
    kh = [x[5] for x in xs]
    lsp = [x[6] for x in xs]
    git = [x[7] for x in xs]
    print(
        f"{name} n={len(xs)} total avg={sum(tots)/len(tots):.1f} max={max(tots):.1f} | "
        f"keydown_handle avg={sum(kh)/len(kh):.2f} max={max(kh):.1f} | "
        f"lsp avg={sum(lsp)/len(lsp):.2f} max={max(lsp):.1f} | "
        f"git avg={sum(git)/len(git):.2f} max={max(git):.1f}"
    )


firsts = []
follows = []
prev_ts = None
for e in ev:
    ts = e[0]
    is_first = prev_ts is None or (ts - prev_ts) > 0.25
    (firsts if is_first else follows).append(e)
    prev_ts = ts

print("\n=== first key after idle (>250ms) vs followup ===")
stats("first-after-idle", firsts)
stats("followup", follows)

print("\n=== dominant component on keydown total>50ms ===")
dom = defaultdict(int)
for r in kd:
    if r["total"] < 50:
        continue
    comps = {k: r[k] for k in comp_keys}
    dom[max(comps, key=comps.get)] += 1
print(dict(dom))

print("\n=== keydown total>100ms detail ===")
dom = defaultdict(int)
for r in sorted(kd, key=lambda r: r["total"], reverse=True):
    if r["total"] < 100:
        continue
    comps = {k: r[k] for k in comp_keys}
    top = max(comps, key=comps.get)
    dom[top] += 1
    print(
        f"frame={r['frame']} total={r['total']:.1f} keydown_handle={r['keydown_handle']:.1f} "
        f"lsp={r['lsp']:.1f} git={r['git']:.1f} acp={r['acp']:.1f} render={r['render']:.1f} "
        f"layout={r['layout_sync']:.1f} preview={r['preview']!r} top={top}"
    )
print("dom", dict(dom))

# sequences: look at consecutive keydown frames and mark first of run
print("\n=== runs of keydowns: first vs rest in burst ===")
runs_first = []
runs_rest = []
prev_frame = None
for r in kd:
    if prev_frame is None or r["frame"] - prev_frame > 15:
        runs_first.append(r)
    else:
        runs_rest.append(r)
    prev_frame = r["frame"]


def avg(xs, key):
    return sum(r[key] for r in xs) / len(xs) if xs else 0


print(
    f"burst-first n={len(runs_first)} total_avg={avg(runs_first,'total'):.1f} "
    f"kh_avg={avg(runs_first,'keydown_handle'):.2f} lsp_avg={avg(runs_first,'lsp'):.2f} "
    f"git_avg={avg(runs_first,'git'):.2f}"
)
print(
    f"burst-rest  n={len(runs_rest)} total_avg={avg(runs_rest,'total'):.1f} "
    f"kh_avg={avg(runs_rest,'keydown_handle'):.2f} lsp_avg={avg(runs_rest,'lsp'):.2f} "
    f"git_avg={avg(runs_rest,'git'):.2f}"
)
