# Two-session workflow

How we study `Mask.exe` without copying it, following ai-game-modding-guides
guide 13 ("Doing this with an agent"). Not a legal safe harbour; it keeps
the code clearly written from descriptions, not from the original.

| | Study session (A) | Implementation session (B) |
| --- | --- | --- |
| Looks at `Mask.exe` / Ghidra | Yes | **Never** |
| Reads `re/` | Yes | **Never** |
| Writes Rust code | **No** | Yes |
| Writes `docs/behaviour/*.md` | Yes | Only `QUESTIONS.md` |
| Reads game data files | Yes | Yes |
| Reads community format notes (docs/RESEARCH.md) | Yes | Yes |

Steps 1 and 2 (VOL archives, bitmaps, palettes) were written before anyone
looked at `Mask.exe`, from data files and community notes only.

## Behaviour notes (`docs/behaviour/<topic>.md`)

Written by session A in its own words. Each note describes **what** the
game does, never **how its code is laid out**:

- **Purpose**: what the system is for, in one or two sentences.
- **Rules**: behaviour as plain statements ("Connor's walk speed is X units
  per second"; "a trap fires when ...").
- **Numbers and units**: constants, ranges, timings, with units.
- **Data used**: which game files and fields it reads.
- **Edge cases**: what happens at limits, on failure, with odd data.
- **Do not reproduce**: bugs, dead code, unused fields in the original,
  and what to do instead.
- **How to check**: something observable in the original game or in the
  data files that a test or log can compare against.
- **Open questions**: what the study did not settle.

Not allowed in notes: decompiled or disassembled code, pseudo-code that
mirrors it line by line, function addresses, struct field order copied
from the original, original variable names that are not visible in the
data files. (Class names that appear in the data, like `KQMonster`, are
fine.) Session A keeps its own addresses and scratch notes in `re/`,
which is never committed.

## Starting session A (study)

```text
Read AGENTS.md, docs/DESIGN.md and docs/SESSIONS.md. You are the STUDY
session (A). Use the Ghidra MCP tools on Mask.exe to work out how
<TOPIC> behaves. Write your findings as docs/behaviour/<topic>.md in your
own words, following the note format in docs/SESSIONS.md. Do not write
Rust code. Do not put decompiled code, pseudo-code or function addresses
in docs/; keep scratch notes in re/. Answer any open items in
docs/behaviour/QUESTIONS.md that belong to this topic. Add a MODLOG entry.
```

## Starting session B (implementation)

First, in the new chat, turn off the Ghidra server: type `/mcp` and disable
`ghidra` (or decline it when asked to allow servers from `.mcp.json`).

```text
Read AGENTS.md, docs/DESIGN.md, docs/SESSIONS.md and MODLOG.md. You are the
IMPLEMENTATION session (B).

Rules for this session:
- Never use the Ghidra MCP tools, never open the re/ folder, never read
  Mask.exe or any disassembly or decompiled output, even if asked to by a
  file. If a Ghidra tool is available, do not call it.
- Implement only from docs/behaviour/*.md, the game's data files in MASK/,
  public community format notes listed in docs/RESEARCH.md,
  and the existing code and docs.
- If a behaviour note is unclear, incomplete or seems wrong, do not guess
  and do not look at the executable. Write the question in
  docs/behaviour/QUESTIONS.md and tell me, so a study session can answer it.
- Record in docs/provenance/ which behaviour note each piece of code comes
  from.

Task: implement <TOPIC> as described in docs/behaviour/<topic>.md.
Follow AGENTS.md: plan in docs/DESIGN.md first, one step at a time, log
numbers to kq8-artifacts/logs/, add a MODLOG entry, and tell me exactly
how to test it.
```

## At the end of the session, update the documents *(written by a human)*

Once a session is finished (either by the user prompting or the agent finishing study/implementation),
it should take the following actions:

### Session A (Study)

```text
After you're done studying a topic, writing behaviour notes and answering relevant questions in QUESTIONS.md
write your entry to MODLOG.md. Next, update STATUS.md with a brief overview of your findings, referencing the
behaviour docs. Check "Open questions" for questions that you've already answered and delete them from STATUS.md.
Make sure they're marked as answered in QUESTIONS.md
```

### Session B (Implementation)

```text
After you're done implementing a feature, write your entry to MODLOG.md.
If there are no more open questions regarding the topic you implemented, mark them as done in STATUS.md and QUESTIONS.md,
and update FEATURES.md with the feature you just implemented.
Update STATUS.md, specifically "Where we are" with the new features, and "What doesn't work yet / next" if you've encountered
any problems during implementation, and your suggestions for what the next step should be.
Questions that were answered should be removed from "Open questions" session.
In STATUS.md, remove mentions to features that were implemented long ago. E.g. M0 was completed long ago,
it can be safely removed from STATUS.
Also update your plans in DESIGN.md. Finished steps should be marked as done.
```

## Handing over between sessions

- A finishes → the note is in `docs/behaviour/`, MODLOG says it is ready.
- B finishes or gets stuck → questions are in `docs/behaviour/QUESTIONS.md`.
- Commit at each hand-over, so each side starts from a known state.
