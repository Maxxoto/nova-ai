#!/usr/bin/env python3
"""DESIGN.md verification — S1 structure, S2 state coverage, S3 token validity."""
import re, sys

PATH = "/home/maxxoto/Project/nova-ai/docs/DESIGN.md"
text = open(PATH, encoding="utf-8").read()
failures, warnings = [], []

# ---------- S1: VoltAgent/Stitch structure ----------
fm = re.match(r"^---\n(.*?)\n---\n", text, re.DOTALL)
if not fm:
    failures.append("S1: no YAML front matter")
else:
    fm_text = fm.group(1)
    try:
        import yaml
        data = yaml.safe_load(fm_text)
        assert isinstance(data, dict)
    except ImportError:
        data, fm_text = None, fm_text
        warnings.append("S1: pyyaml missing — regex fallback")
    except Exception as e:
        failures.append(f"S1: YAML parse error: {e}")
        data = None
    if data:
        for key in ["version", "name", "description", "colors", "typography",
                    "rounded", "spacing", "motion", "components"]:
            if key not in data:
                failures.append(f"S1: front matter missing '{key}'")
        print(f"S1 front matter keys: {sorted(data.keys())}")
        print(f"S1 token counts: colors={len(data['colors'])} typography={len(data['typography'])} "
              f"rounded={len(data['rounded'])} spacing={len(data['spacing'])} "
              f"motion={len(data['motion'])} components={len(data['components'])}")

required_sections = ["Overview", "## Colors", "## Typography", "## Layout",
                     "Elevation & Depth", "## Shapes", "AI State System", "## Components",
                     "Do's and Don'ts", "Responsive", "shadcn Implementation",
                     "Agent Prompt Guide", "Iteration Guide"]
for sec in required_sections:
    if sec not in text:
        failures.append(f"S1: missing section '{sec}'")

# ---------- S2: state coverage + no placeholders ----------
required_states = ["idle", "listening", "transcribing", "thinking", "speaking",
                   "streaming", "complete", "error", "degraded",
                   "offline", "local-only", "local_only", "calling-cloud", "calling_cloud"]
for st in required_states:
    if st not in text.lower():
        failures.append(f"S2: state '{st}' not covered")

for ph in ["TBD", "TODO", "FIXME", "placeholder", "lorem", "XXX"]:
    hits = [ln for ln in text.splitlines() if ph.lower() in ln.lower()
            and not ln.strip().startswith(">")]
    if hits:
        failures.append(f"S2: placeholder '{ph}' found: {hits[:2]}")

prd_trace = ["F-01", "F-02", "F-03", "F-04", "F-05", "F-06", "F-07", "F-08",
             "F-09", "F-10", "F-11", "F-12", "F-13", "F-14", "AC-01", "AC-02", "AC-03",
             "AC-06", "AC-08", "AC-09", "AC-10", "AC-11", "AC-12"]
for t in prd_trace:
    if t not in text:
        warnings.append(f"S2: PRD/RFC trace '{t}' not referenced")

# ---------- S3: token ref resolution ----------
token_prefixes = ["colors", "typography", "rounded", "spacing", "motion", "components"]
if data:
    refs = set(re.findall(r"\{((?:colors|typography|rounded|spacing|motion|components)\.[a-z0-9-]+)\}", text))
    undefined = []
    for ref in refs:
        parts = ref.split(".", 1)
        container = data.get(parts[0], {})
        if isinstance(container, dict) and parts[1] not in container:
            undefined.append(ref)
    if undefined:
        failures.append(f"S3: undefined token refs: {sorted(undefined)}")
    else:
        print(f"S3: all {len(refs)} token refs resolve")

# ---------- S3: hex validity + AA contrast ----------
def hex_rgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i+2], 16) for i in (0, 2, 4)) if len(h) == 6 else None

def lum(rgb):
    def ch(c):
        c /= 255
        return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4
    r, g, b = map(ch, rgb)
    return 0.2126 * r + 0.7152 * g + 0.0722 * b

def contrast(fg, bg):
    l1, l2 = lum(hex_rgb(fg)), lum(hex_rgb(bg))
    hi, lo = max(l1, l2), min(l1, l2)
    return (hi + 0.05) / (lo + 0.05)

if data:
    bad_hex = [k for k, v in data["colors"].items()
               if isinstance(v, str) and v.startswith("#") and not hex_rgb(v)]
    if bad_hex:
        failures.append(f"S3: invalid hex: {bad_hex}")
    pairs = [
        ("ink on canvas", "ink", "canvas", 4.5), ("body on canvas", "body", "canvas", 4.5),
        ("mute on canvas", "mute", "canvas", 4.5),
        ("white on primary", "primary-foreground", "primary", 4.5),
        ("ink on surface", "ink", "surface", 4.5),
        ("accent-fg on accent", "accent-foreground", "accent", 4.5),
        # secondary = thinking/processing state graphic (DESIGN.md), not a text surface — check real usages:
        ("secondary vs canvas (state graphic, non-text)", "secondary", "canvas", 3.0),
        ("secondary-fg on canvas (indigo state text)", "secondary-foreground", "canvas", 4.5),
        ("dark-secondary-fg on dark-secondary (night-sky fill)", "dark-secondary-foreground", "dark-secondary", 4.5),
        ("warning on canvas", "warning", "canvas", 4.5),
        ("success on canvas", "success", "canvas", 4.5),
        ("destructive on canvas", "destructive", "canvas", 4.5),
        ("dark-ink on dark-canvas", "dark-ink", "dark-canvas", 4.5),
        ("dark-primary-fg on dark-primary", "dark-primary-foreground", "dark-primary", 4.5),
        ("dark-body on dark-canvas", "dark-body", "dark-canvas", 4.5),
        ("dark-mute on dark-canvas", "dark-mute", "dark-canvas", 3.0),
        ("live vs canvas (non-text)", "live", "canvas", 3.0),
        ("dark-live vs dark-canvas (non-text)", "dark-live", "dark-canvas", 3.0),
    ]
    for name, fg, bg, need in pairs:
        c = contrast(data["colors"][fg], data["colors"][bg])
        status = "PASS" if c >= need else "FAIL"
        print(f"S3 contrast {name}: {c:.2f}:1 (need {need}) {status}")
        if c < need:
            failures.append(f"S3: contrast {name} = {c:.2f} < {need}")

# ---------- S3b: CSS bridge ↔ front-matter token cross-check ----------
def hsl_to_hex(h, s_pct, l_pct):
    s, l = s_pct / 100, l_pct / 100
    c = (1 - abs(2 * l - 1)) * s
    hp = (h % 360) / 60
    x = c * (1 - abs(hp % 2 - 1))
    r1, g1, b1 = [(c, x, 0), (x, c, 0), (0, c, x), (0, x, c), (x, 0, c), (c, 0, x)][int(hp) % 6]
    m = l - c / 2
    return "#%02x%02x%02x" % tuple(round((v + m) * 255) for v in (r1, g1, b1))

css = re.search(r"```css\n(.*?)```", text, re.DOTALL)
if not css:
    failures.append("S3b: no css bridge block found")
elif data:
    bridge = {"light": {}, "dark": {}}
    theme = None
    for line in css.group(1).splitlines():
        line = line.split("/*")[0].strip().rstrip(";").strip()
        if line.startswith(":root"):
            theme = "light"
        elif line.startswith(".dark"):
            theme = "dark"
        m = re.match(r"--([a-z-]+):\s*([\d.]+)\s+([\d.]+)%\s+([\d.]+)%$", line)
        if m and theme:
            bridge[theme][m.group(1)] = hsl_to_hex(float(m.group(2)), float(m.group(3)), float(m.group(4)))
    # var → (light token, dark token)
    mapping = {
        "background": ("canvas", "dark-canvas"), "foreground": ("ink", "dark-ink"),
        "card": ("surface", "dark-surface"), "card-foreground": ("ink", "dark-ink"),
        "popover": ("surface", "dark-surface"), "popover-foreground": ("ink", "dark-ink"),
        "primary": ("primary", "dark-primary"), "primary-foreground": ("primary-foreground", "dark-primary-foreground"),
        "primary-soft": ("primary-soft", "dark-primary-soft"), "primary-active": ("primary-active", "dark-primary-active"),
        "thinking-text": ("thinking-text", "dark-thinking"), "speaking-text": ("primary", "dark-speaking"),
        "secondary": ("secondary", "dark-secondary"), "secondary-foreground": ("secondary-foreground", "dark-secondary-foreground"),
        "muted": ("surface-soft", "dark-surface-soft"), "muted-foreground": ("mute", "dark-mute"),
        "accent": ("accent", "dark-accent"), "accent-foreground": ("accent-foreground", "dark-accent-foreground"),
        "destructive": ("destructive", "dark-destructive"), "destructive-foreground": ("destructive-foreground", "dark-destructive-foreground"),
        "success": ("success", "dark-success"), "warning": ("warning", "dark-warning"),
        "live": ("live", "dark-live"), "border": ("border", "dark-border"), "input": ("border", "dark-border"),
        "ring": ("primary", "dark-primary"),
    }
    missing_vars = [v for v in mapping if v not in bridge["light"] or v not in bridge["dark"]]
    if missing_vars:
        failures.append(f"S3b: bridge missing vars: {missing_vars}")
    extra_vars = (set(bridge["light"]) | set(bridge["dark"])) - set(mapping)
    if extra_vars:
        failures.append(f"S3b: bridge vars with no token mapping: {sorted(extra_vars)}")
    for var, (lt, dt) in mapping.items():
        for theme, tok in (("light", lt), ("dark", dt)):
            if var in bridge[theme] and tok in data["colors"]:
                want, got = data["colors"][tok].lower(), bridge[theme][var].lower()
                if want != got:
                    failures.append(f"S3b: .{theme} --{var} = {bridge[theme][var]} ≠ {tok} {want}")

# ---------- S3c: extended contrast pairs (banners, chips, state text) ----------
if data:
    c = data["colors"]
    extra_pairs = [
        ("destructive on primary-soft (banner-error)", "destructive", "primary-soft", 4.5),
        ("warning on primary-soft (banner-degraded)", "warning", "primary-soft", 4.5),
        ("primary-active on primary-soft (citation chip)", "primary-active", "primary-soft", 4.5),
        ("thinking-text on canvas", "thinking-text", "canvas", 4.5),
        ("dark-destructive on dark-surface-soft (banner)", "dark-destructive", "dark-surface-soft", 4.5),
        ("dark-warning on dark-surface-soft (banner)", "dark-warning", "dark-surface-soft", 4.5),
        ("dark-destructive on dark-canvas", "dark-destructive", "dark-canvas", 4.5),
        ("dark-warning on dark-canvas", "dark-warning", "dark-canvas", 4.5),
        ("dark-primary-active on dark-primary-soft (chip)", "dark-primary-active", "dark-primary-soft", 4.5),
        ("dark-thinking on dark-canvas (state text)", "dark-thinking", "dark-canvas", 4.5),
        ("white on destructive", "destructive-foreground", "destructive", 4.5),
        ("dark-destructive-foreground on dark-destructive", "dark-destructive-foreground", "dark-destructive", 4.5),
    ]
    for name, fg, bg, need in extra_pairs:
        cv = contrast(c[fg], c[bg])
        status = "PASS" if cv >= need else "FAIL"
        print(f"S3c contrast {name}: {cv:.2f}:1 (need {need}) {status}")
        if cv < need:
            failures.append(f"S3c: contrast {name} = {cv:.2f} < {need}")

# ---------- report ----------
print("\nWarnings:", *warnings, sep="\n  ") if warnings else print("Warnings: none")
if failures:
    print("\nFAILURES:", *failures, sep="\n  ")
    sys.exit(1)
print("\nALL CHECKS PASSED")
