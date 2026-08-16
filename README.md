# CLI Task Manager

A simple CLI task manager written in Rust with local JSON storage.

## Usage

```text
task-manager add <title...>
task-manager list
task-manager list --status <todo|in-progress|done>
task-manager delete <id>
task-manager update <id> <todo|in-progress|done>
```

`add` joins all remaining arguments into one title. Every other command rejects
trailing arguments rather than ignoring them. Status arguments are matched
case-insensitively and trimmed.

## Data file

Location: `./tasks.json`, relative to the working directory (project root during development).
Created on first save. Missing file = empty task list, not an error.

```
{
  "version": 1,
  "next_id": 3,
  "tasks": [
    { "id": 1, "title": "Write README", "status": "done" },
    { "id": 2, "title": "Implement add", "status": "in-progress" }
  ]
}
```

- `next_id` only ever increments; IDs are never reused after delete.
- `status` is one of: `todo`, `in-progress`, `done`.

## Features

- [x] add
- [x] list
- [x] list --status
- [x] delete
- [x] update
- [ ] stats

Commands are wired end-to-end; persistence lands on Day 5, so nothing survives
between runs yet.

## Layout

One line per file that holds a decision. Generated files, lockfiles, and data
fixtures are not listed. Sizes are deliberately absent - they go stale; only
threshold exceptions are recorded.

Rule: each responsibility is one sentence with no "and". A line that needs an
"and" is a split waiting to happen, and stays on this list as a known debt.

| Path           | Responsibility (one sentence, no "and")                         | Threshold                                                            |
| -------------- | --------------------------------------------------------------- | -------------------------------------------------------------------- |
| `src/lib.rs`   | Defines the task data types, their error type, and their rules. |                                                                      |
| `src/tests.rs` | Holds the unit tests for the library.                           | 800 - one test per behaviour; grouping them elsewhere hides coverage |
| `src/main.rs`  | Parses CLI arguments, then hands off to the library.            |                                                                      |

Threshold column: leave blank for the default. Fill it only when a file has an
approved exception, always with the reason.

### Known debts

- `src/lib.rs` - responsibility needs an "and": defines the data types and
  applies the command rules. Split at the types/logic seam if the file crosses
  ~350 lines. Left alone for now because the types and the rules that guard
  their invariants change together, and separating them would scatter one
  contract across two files.
- `src/main.rs` - the `run_*` functions live in the binary, so `src/tests.rs`
  cannot reach them. Their argument-shape rules (positional slots, trailing-arg
  rejection) are therefore unverified. Either move the parsing into the library
  or add an inline test module here once the commands do something worth
  asserting.

## Project-wide invariants

- `Status` renders through `Display` exactly as it serializes to JSON: `todo`, `in-progress`, `done`.
- `Status` has two parsers and they deliberately differ. `FromStr` is the
  lenient CLI-facing one: it trims and lowercases, so `" DONE "` parses to
  `done`. serde is the strict file-facing one: `tasks.json` must contain exact
  kebab-case (`todo`, `in-progress`, `done`) or the load fails. Data-file
  parsing never routes through `FromStr`, and `FromStr` errors always echo the
  caller's original input, not the normalised form.
- `Task`'s `Display` is human-facing only and is never parsed.
- Only `version: 1` is accepted. Any other value is a hard error: refuse to
  load, exit without writing the data file.
- `TaskStore::next_id` is the only source of task IDs. `TaskStore::add_task` is
  the only path that allocates one, and it bumps `next_id` only after the task
  is successfully created. `Task::new` takes an `id` from its caller and does
  not check it — calling it directly outside `add_task` can produce duplicate
  IDs and is not supported.
- `list_tasks` and `list_by_status` return tasks in insertion order, which is
  ascending `id` order. Display order is never re-sorted.
