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

## Project-wide invariants

- `Status` renders through `Display` exactly as it serializes to JSON: `todo`, `in-progress`, `done`.
- `Task`'s `Display` is human-facing only and is never parsed.
