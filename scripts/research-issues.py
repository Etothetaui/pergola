#!/usr/bin/env python3
"""Harvest early bug reports from other tiling window managers.

The evidence base behind `docs/HISTORY-LESSONS.md`: pulls the first years of
issues from six projects over the GitHub REST API (no token needed for this
volume), clusters them by keyword into failure classes, and writes a digest.

    scripts/research-issues.py --out /tmp/pergola-research

Search engines are bot-walled on this host, so the GitHub API is the interface:
https://api.github.com/repos/<owner>/<repo>/issues?state=all&per_page=100&page=N
An unauthenticated client gets 60 requests/hour; six repos need ~30. Set
GITHUB_TOKEN to raise the limit.

This script is a guard, not a dependency: it is run by hand when the lesson
document is revised, and its output is what the document cites.
"""

import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.request
from collections import defaultdict

# repo -> last day of "early versions" that still teaches us something.
REPOS = {
    "i3/i3": "2013-12-31",
    "awesomeWM/awesome": "2016-12-31",
    "baskerville/bspwm": "2017-12-31",
    "xmonad/xmonad": "2013-12-31",
    "herbstluftwm/herbstluftwm": "2017-12-31",
    "qtile/qtile": "2018-12-31",
}

# class name -> keywords; an issue may land in several classes on purpose.
CLASSES = {
    "keymap / keyboard layout": ["keymap", "layout", "xkb", "neo", "bone", "dvorak",
                                 "cyrillic", "russian", "altgr", "keysym", "keyboard"],
    "hotkey grabs": ["grab", "binding", "shortcut", "hotkey", "keybind", "numlock",
                     "capslock", "modifier", "keys"],
    "multi-monitor / randr": ["monitor", "randr", "xrandr", "screen", "output",
                              "display", "hotplug", "dpi"],
    "focus / raise": ["focus", "raise", "urgent", "steal", "activate"],
    "geometry / size hints": ["resize", "size hint", "geometry", "width", "height",
                              "pixel", "increment", "clamp"],
    "fullscreen / maximize": ["fullscreen", "maximize", "maximized", "f11"],
    "transient / dialog": ["dialog", "transient", "popup", "modal", "splash"],
    "crash / hang": ["crash", "segfault", "hang", "freeze", "panic", "deadlock",
                     "abort", "assert"],
    "memory / cpu": ["memory leak", "cpu", "leak", "ram", "100%"],
    "restart / state": ["restart", "reload", "state", "session", "persist"],
    "config / errors": ["config", "parse", "invalid", "error message", "validation"],
    "startup / empty": ["startup", "start", "empty", "no output", "login"],
    "ewmh / icccm": ["ewmh", "icccm", "net_wm", "motif", "wm_class", "property",
                     "atom"],
    "compositing": ["composit", "opacity", "transparent", "blur", "picom", "compton"],
    "tray / panel": ["tray", "systray", "panel", "bar", "status"],
    "workspace / tag": ["workspace", "tag", "desktop", "virtual"],
    "ipc / scripting": ["ipc", "socket", "script", "command", "subscribe"],
    "build / packaging": ["build", "compile", "makefile", "package", "install",
                          "dependency"],
    "drag / resize": ["drag", "drag and drop", "pointer", "cursor", "mouse"],
    "lock / screensaver": ["lock", "screensaver", "i3lock", "xscreensaver"],
    "title property": ["title", "name", "caption"],
    "window type": ["window type", "type", "toolbar", "utility", "dock"],
    "performance": ["slow", "lag", "performance", "latency"],
    "stacking": ["stack", "above", "below", "obscure", "z-order", "overlap"],
    "window classes": ["class", "instance", "wm_class"],
}


def api(path):
    url = "https://api.github.com" + path
    req = urllib.request.Request(url)
    req.add_header("Accept", "application/vnd.github+json")
    req.add_header("User-Agent", "pergola-research")
    token = os.environ.get("GITHUB_TOKEN")
    if token:
        req.add_header("Authorization", "Bearer " + token)
    with urllib.request.urlopen(req, timeout=60) as r:
        return json.load(r), r.headers


def harvest(repo, until):
    """Every issue in the repo created on or before `until`."""
    out, page = [], 1
    while True:
        items, headers = api(
            f"/repos/{repo}/issues?state=all&per_page=100&page={page}&sort=created&direction=asc"
        )
        if not items:
            break
        stop = False
        for it in items:
            if "pull_request" in it:
                continue
            if it["created_at"][:10] > until:
                stop = True
                break
            out.append(
                {
                    "number": it["number"],
                    "state": it["state"],
                    "title": it["title"],
                    "created": it["created_at"][:10],
                    "comments": it["comments"],
                    "body": (it.get("body") or "")[:1500],
                }
            )
        if stop or len(items) < 100:
            break
        page += 1
        time.sleep(0.4)
        if page > 20:  # 2000 issues is plenty for a lesson document
            break
    return out


def classify(issues):
    classes = defaultdict(list)
    for repo, items in issues.items():
        for it in items:
            text = (it["title"] + " " + it["body"]).lower()
            for name, words in CLASSES.items():
                if any(w in text for w in words):
                    classes[name].append((repo, it))
    return classes


def digest(issues, classes):
    lines = []
    for repo, items in issues.items():
        lines.append(f"## {repo}\n")
        for it in items:
            body = " ".join(it["body"].split())[:300]
            lines.append(
                f"### #{it['number']} [{it['state']}] {it['title']}  ({it['comments']}c)\n"
                f"{body}\n"
            )
    lines.append("\n# Failure classes\n")
    for name in sorted(classes, key=lambda n: -len(classes[n])):
        rows = classes[name]
        lines.append(f"## {name} ({len(rows)})\n")
        for repo, it in rows:
            lines.append(f"- {repo}#{it['number']} [{it['state']}] {it['title'][:90]}")
        lines.append("")
    return "\n".join(lines)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="/tmp/pergola-research")
    args = ap.parse_args()
    os.makedirs(args.out, exist_ok=True)
    issues = {}
    for repo, until in REPOS.items():
        print(f"harvesting {repo} (until {until})", file=sys.stderr)
        try:
            issues[repo] = harvest(repo, until)
        except urllib.error.HTTPError as e:
            print(f"  {repo}: HTTP {e.code} — skipped", file=sys.stderr)
            issues[repo] = []
        with open(os.path.join(args.out, repo.replace("/", "-") + ".json"), "w") as f:
            json.dump({"issues": issues[repo]}, f, indent=1)
    classes = classify(issues)
    with open(os.path.join(args.out, "digest.md"), "w") as f:
        f.write(digest(issues, classes))
    with open(os.path.join(args.out, "classes.txt"), "w") as f:
        for name in sorted(classes, key=lambda n: -len(classes[n])):
            f.write(f"## {name} ({len(classes[name])})\n")
            for repo, it in classes[name]:
                f.write(f"- {repo}#{it['number']} [{it['state']}] {it['title'][:90]}\n")
            f.write("\n")
    total = sum(len(v) for v in issues.values())
    print(f"{total} issues, {len(classes)} classes -> {args.out}", file=sys.stderr)


if __name__ == "__main__":
    main()
