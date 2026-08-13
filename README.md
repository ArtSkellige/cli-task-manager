# CLI Task Manager

A simple CLI task manager written in Rust with local JSON storage.

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

- [ ] add
- [ ] list
- [ ] list --status
- [ ] delete
- [ ] stats

## Layout

Single file: `src/main.rs` defines the data types. Command logic and JSON
read/write will land in the same file as the roadmap progresses, at which point
it holds three responsibilities - see Known debts.

### Known debts

- `src/main.rs` - will need "and" by Day 5: defines the data types, applies
  command rules, and reads/writes the data file. Split at the IO/logic seam
  during the Day 7 refactor, or sooner if the file crosses ~350 lines. Left
  alone for now because module paths and visibility aren't concepts I've
  practised yet.

## Project-wide invariants

- `Status` renders through `Display` exactly as it serializes to JSON: `todo`, `in-progress`, `done`.
- `Task`'s `Display` is human-facing only and is never parsed.
