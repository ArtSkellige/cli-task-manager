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

Single file: `src/main.rs` defines the data types and applies the command rules
(`add_task`, `list_tasks`, `list_by_status`). JSON read/write lands in the same
file on Day 5, giving it three responsibilities - see Known debts.

### Known debts

- `src/main.rs` - already needs an "and": defines the data types and applies
  command rules. Reads/writes the data file from Day 5. Split at the IO/logic
  seam during the Day 7 refactor, or sooner if the file crosses ~350 lines. Left
  alone for now because module paths and visibility aren't concepts I've
  practised yet.

## Project-wide invariants

- `Status` renders through `Display` exactly as it serializes to JSON: `todo`, `in-progress`, `done`.
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
