#!/usr/bin/env python3
"""Check the Markdown documentation against the code and itself.

    scripts/doc-check.py            check every Markdown file
    scripts/doc-check.py --list     print what each check covers

What is checked, and why each one has bitten us before:

* every relative link and image resolves (a renamed doc leaves a 404 on GitHub);
* every fenced ```json block parses, unless it is marked illustrative with a
  literal `…` (the config examples are the ones users copy);
* every configuration table lists exactly the keys `pergola config` knows
  (`KNOWN_KEYS` in config.rs) — the README table silently lost `notify`;
* every `pergola <command>` named in prose exists in the CLI or the daemon, and
  every `pergola config <key>` names a real key.

Exit status is 1 on any finding, so a timer or a release step can gate on it.
This is deliberately a *reader* of the sources, not a second copy of the
tables: the only lists here are the files to scan and the spelling of the
Markdown constructs.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# Markdown outside the build tree. BACKLOG/CHANGELOG are included: they link
# into docs/ too, and a broken link there is still a broken link.
SKIP_DIRS = {".git", "target", "vendor", "node_modules"}


def markdown_files() -> list[Path]:
    out = []
    for path in ROOT.rglob("*.md"):
        if any(part in SKIP_DIRS for part in path.relative_to(ROOT).parts):
            continue
        out.append(path)
    return sorted(out)


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT))


# --- links ---------------------------------------------------------------

LINK = re.compile(r"!?\[[^\]]*\]\(([^)]+)\)")
SKIP_SCHEMES = ("http://", "https://", "mailto:", "pergola://", "ftp://")


def check_links(paths: list[Path]) -> list[str]:
    problems = []
    for path in paths:
        for target in LINK.findall(read(path)):
            target = target.strip()
            if target.startswith("<") and ">" in target:
                target = target[1 : target.index(">")]
            if not target or target.startswith("#") or target.startswith(SKIP_SCHEMES):
                continue
            # Placeholders a reader is meant to substitute are not links, and
            # a destination with a space is not a Markdown link at all (the
            # issue digest quotes report bodies in brackets).
            if any(ch in target for ch in "<>{}…*") or " " in target:
                continue
            target = target.split("#", 1)[0].split("?", 1)[0]
            if not target:
                continue
            resolved = (path.parent / target).resolve()
            if not resolved.exists():
                problems.append(f"{rel(path)}: link target does not exist: {target}")
    return problems


# --- JSON examples -------------------------------------------------------

FENCE = re.compile(r"^```(\w*)\s*$")


def json_blocks(text: str) -> list[tuple[int, str]]:
    """(1-based line of the opening fence, body) for every ```json block."""
    blocks = []
    lines = text.splitlines()
    i = 0
    while i < len(lines):
        m = FENCE.match(lines[i])
        if m and m.group(1) == "json":
            start = i + 1
            body = []
            i += 1
            while i < len(lines) and not FENCE.match(lines[i]):
                body.append(lines[i])
                i += 1
            blocks.append((start, "\n".join(body)))
        i += 1
    return blocks


def check_json(paths: list[Path]) -> tuple[list[str], int]:
    problems, skipped = [], 0
    for path in paths:
        for line, body in json_blocks(read(path)):
            # An ellipsis is the Markdown convention for "rest of the object";
            # those blocks are illustrations, not copyable examples.
            if "…" in body or "..." in body:
                skipped += 1
                continue
            try:
                value = json.loads(body)
            except json.JSONDecodeError as e:
                problems.append(f"{rel(path)}:{line}: invalid JSON: {e}")
                continue
            if not isinstance(value, dict):
                problems.append(f"{rel(path)}:{line}: JSON example is not an object")
    return problems, skipped


# --- configuration keys --------------------------------------------------

KNOWN_KEYS = re.findall(
    r'"([a-z_]+)",',
    re.search(
        r"pub const KNOWN_KEYS: &\[&str\] = &\[(.*?)\];",
        read(ROOT / "crates" / "pergola" / "src" / "config.rs"),
        re.S,
    ).group(1),
)

TABLE_ROW = re.compile(r"^\|(.+)\|\s*$")


def tables(text: str) -> list[list[list[str]]]:
    """Every pipe table as a list of rows of stripped cells."""
    found, rows = [], []
    for line in text.splitlines() + [""]:
        m = TABLE_ROW.match(line)
        if m:
            rows.append([c.strip() for c in m.group(1).split("|")])
            continue
        if len(rows) >= 2:
            found.append(rows)
        rows = []
    return found


def check_config_tables(paths: list[Path]) -> list[str]:
    problems = []
    for path in paths:
        for rows in tables(read(path)):
            header = [c.lower() for c in rows[0]]
            if "default" not in header:
                continue
            keys = []
            for row in rows[2:]:
                if not row or not row[0].startswith("`"):
                    continue
                key = row[0].strip("`").strip()
                if re.fullmatch(r"[a-z_]+", key):
                    keys.append(key)
            # Only a table that is mostly configuration keys is the config
            # table; a table of environment variables is not.
            known = [k for k in keys if k in KNOWN_KEYS]
            if len(known) < 3:
                continue
            missing = [k for k in KNOWN_KEYS if k not in keys]
            unknown = [k for k in keys if k not in KNOWN_KEYS]
            if missing:
                problems.append(
                    f"{rel(path)}: configuration table omits: {', '.join(missing)}"
                )
            if unknown:
                problems.append(
                    f"{rel(path)}: configuration table documents unknown keys: {', '.join(unknown)}"
                )
    return problems


# --- command names -------------------------------------------------------

ARM = re.compile(r'^\s*"([a-z][a-z0-9_-]*)"\s*=>', re.M)


def source_commands() -> set[str]:
    names: set[str] = set()
    for path in (
        ROOT / "crates" / "pergola" / "src" / "main.rs",
        ROOT / "crates" / "pergola" / "src" / "daemon.rs",
        ROOT / "crates" / "pergola-core" / "src" / "command.rs",
    ):
        names |= set(ARM.findall(read(path)))
    # Aliases and the two words the CLI resolves before dispatch.
    names |= {"help", "version", "--help", "--version", "-h", "-V"}
    return names


# Only code is treated as a claim about the CLI: prose like "Pergola is ..."
# and shell like `cd pergola` must not be read as command names.
CODE_SPAN = re.compile(r"`([^`\n]+)`")
DOC_COMMAND = re.compile(r"(?:^|[|;$][ \t]*|\$[ \t]+)pergola[ \t]+([a-z][a-z0-9_-]*)", re.M)


def code_text(text: str) -> str:
    """Inline code spans and fenced blocks, one per line."""
    chunks = CODE_SPAN.findall(text)
    lines = text.splitlines()
    i = 0
    while i < len(lines):
        m = FENCE.match(lines[i])
        if m:
            body = []
            i += 1
            while i < len(lines) and not FENCE.match(lines[i]):
                body.append(lines[i])
                i += 1
            chunks.append("\n".join(body))
        i += 1
    return "\n".join(chunks)


def check_commands(paths: list[Path]) -> list[str]:
    known = source_commands()
    problems = []
    for path in paths:
        code = code_text(read(path))
        for name in DOC_COMMAND.findall(code):
            # `pergola config <key>` is checked separately; the bare word
            # "config" is a command either way.
            if name not in known:
                problems.append(f"{rel(path)}: documents unknown command: pergola {name}")
        for key in re.findall(r"pergola config ([a-z_]+)", code):
            if key not in KNOWN_KEYS:
                problems.append(f"{rel(path)}: pergola config names unknown key: {key}")
    return problems


# --- keyboard table ------------------------------------------------------

BINDING_TUPLE = re.compile(r'\("([^"]+)", vec!\[([^\]]*)\]\)')


def check_bindings() -> tuple[list[str], int]:
    """Every default chord in binding_plan() must appear in KEYBINDINGS.md.

    The point is to catch a binding added or changed in code while the table a
    reader trusts still shows the old chord. The table may show more than the
    plan (examples, tag chords), never less.
    """
    source = read(ROOT / "crates" / "pergola-x11" / "src" / "lib.rs")
    body = re.search(r"pub fn binding_plan\(\) -> .*?\n\}\n", source, re.S)
    if not body:
        return (["doc-check: could not find binding_plan() in pergola-x11"], 0)
    plan = BINDING_TUPLE.findall(body.group(0))
    if not plan:
        return (["doc-check: binding_plan() has no tuples; parser is stale"], 0)
    doc = read(ROOT / "docs" / "KEYBINDINGS.md")
    problems = []
    for command, chords in plan:
        for chord in re.findall(r'"([^"]+)"', chords):
            if chord not in doc:
                problems.append(
                    f"docs/KEYBINDINGS.md: default chord not documented: {chord} ({command})"
                )
    return problems, sum(len(re.findall(r'"[^"]+"', c)) for _, c in plan)


def main() -> int:
    paths = markdown_files()
    if "--list" in sys.argv:
        for path in paths:
            print(rel(path))
        return 0

    problems = []
    problems += check_links(paths)
    json_problems, json_skipped = check_json(paths)
    problems += json_problems
    problems += check_config_tables(paths)
    problems += check_commands(paths)
    binding_problems, binding_count = check_bindings()
    problems += binding_problems

    for problem in problems:
        print(problem)
    print(
        f"doc-check: {len(paths)} files, {len(KNOWN_KEYS)} config keys, "
        f"{len(source_commands())} command spellings, "
        f"{binding_count} default chord(s) checked, "
        f"{len(problems)} problem(s); {json_skipped} illustrative JSON block(s) skipped"
    )
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
