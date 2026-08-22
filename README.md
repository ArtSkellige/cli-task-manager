# CLI Task Manager

A simple CLI task manager written in Rust with local JSON storage.

## Usage

```text
task-manager add <title...>
task-manager list
task-manager list --status <todo|in-progress|done>
task-manager delete <id>
task-manager update <id> <todo|in-progress|done>
task-manager fetch <url>
```

`add` joins all remaining arguments into one title. Every other command rejects
trailing arguments rather than ignoring them. Status arguments are matched
case-insensitively and trimmed.

`fetch` requests the URL, extracts the text between `<title>` and `</title>`,
collapses interior whitespace to single spaces, and adds the result as a new
`todo` task. Tag matching is ASCII-case-insensitive and skips attributes.
No `<title>` tag, or one with no non-whitespace content, falls back to the
URL itself as the title.
HTML entities are not decoded: a page titled `Rust &amp; Go` becomes a task
titled literally `Rust &amp; Go`. Requests time out after 10 seconds total,
covering connection and body read.

## Data file

Location: `./tasks.json`, relative to the working directory (project root during development).
Created on first save. Missing file = empty task list, not an error.

Writes are atomic: `save()` serialises to a sibling temp file (the target path
with `.tmp` appended, so `./tasks.json.tmp` by default), then renames it over
the target. The temp file is always in the same directory as the target, which
is what makes the rename atomic.

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
- [x] fetch

Commands are wired end-to-end and persist to `./tasks.json` on every mutation.

## Layout

Each file gets one sentence with no "and". A line needing an "and" holds more
than one responsibility and appears under Known debts. Line counts are deliberately absent -
they go stale, and they were never the thing that mattered.

| Path           | Responsibility                                                        | Threshold                                                            |
| -------------- | --------------------------------------------------------------------- | -------------------------------------------------------------------- |
| `src/lib.rs`   | Defines the task data types, their rules, and their JSON persistence. | 500                                                                  |
| `src/tests.rs` | Holds the unit tests for the library.                                 | 800 - one test per behaviour; grouping them elsewhere hides coverage |
| `src/main.rs`  | Parses CLI arguments, fetches URLs, then hands off to the library.    | 500                                                                  |

### Known debts

- `src/lib.rs` - responsibility needs an "and", now three times over: defines
  the data types, applies the command rules, and owns file I/O. The strongest
  seam is IO vs logic - `load`/`save` plus the path constants and the
  `ReadFailed`/`WriteFailed`/`Json` variants could move to `src/store.rs`.
  Left alone at ~250 lines because the persistence code is thirty lines and
  splitting now would put `TaskStore` and two of its methods in different
  files. Revisit if the file crosses ~350 lines or if persistence grows a
  second concern (backups, migrations, a configurable path).
- `TaskStore::load_from` and `save_to` are `pub` only so tests can supply a
  scratch path; nothing outside the crate should call them. `load`/`save` are
  the real entry points. Revisit if the crate ever gets external consumers.
- `src/main.rs` - responsibility needs an "and": it parses CLI arguments and
  owns the outbound HTTP call plus the HTML title scrape. The seam is layer of
  abstraction - `extract_title` is pure logic with no IO, and `run_fetch` is
  the only IO around it. Left alone because moving `extract_title` to
  `src/lib.rs` was tried and reverted: a task library that knows about
  `<title>` tags has absorbed its caller's problem, and that cost outranks the
  one-sentence rule here. Revisit if a second command needs network access, at
  which point `src/fetch.rs` holds both.
- `extract_title` lives in the binary, so `src/tests.rs` cannot reach it. It is
  the only pure function in the fetch path and the only one worth unit-testing.
  Covered by an inline `#[cfg(test)]` module in `src/main.rs` instead, which
  splits the project's tests across two locations. Accepted: coverage of a
  pure function outranks a single test location.
- `TaskError::FetchFailed(reqwest::Error)` puts an HTTP crate in the library's
  public error surface, even though the library itself never makes a request -
  `run_fetch` lives in `src/main.rs`. The alternative is `FetchFailed(String)`,
  which drops `reqwest` from the lib's conceptual API. Kept as
  `reqwest::Error` because it preserves the `Error::source()` chain, so the
  real diagnostic - DNS failure vs. TLS failure vs. connection refused -
  survives to the caller instead of being flattened into a message. The
  coupling is paid at the workspace level regardless. Revisit if the library
  ever ships as a standalone crate for external consumers.

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
- Missing `./tasks.json` is not an error: `load()` returns an empty store with
  `version: 1`, `next_id: 1`. Only `ErrorKind::NotFound` is treated this way;
  any other read failure is a hard error.
- Both `Task` and `TaskStore` use `serde(deny_unknown_fields)`. This means an
  unknown key fails as a JSON parse error before the `version` check runs, so a
  future v2 file carrying a new field reports "not valid JSON" rather than the
  friendlier `FileVersionTooNew`. Accepted: correctness of the v1 contract
  outranks the quality of a message for a file format that does not exist yet.
  Fix by parsing `version` in a first pass if v2 ever ships.
