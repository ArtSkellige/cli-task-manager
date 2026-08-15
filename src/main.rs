use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

// TODO: Day 5 - enforce on load; only version 1 is accepted.
#[derive(Debug)]
pub enum TaskError {
    FileVersionTooNew { found: u32 },
    FileVersionInvalid { found: u32 },
    EmptyTitle,
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskError::FileVersionTooNew { found } => write!(
                f,
                "task file was written by a newer version of this app (file version {found}); \
 update the app to open it"
            ),
            TaskError::FileVersionInvalid { found } => write!(
                f,
                "task file is corrupt or hand-edited (invalid version {found})"
            ),
            TaskError::EmptyTitle => write!(f, "task title cannot be empty"),
        }
    }
}

impl Error for TaskError {}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub status: Status,
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {} ({})", self.id, self.title, self.status)
    }
}

impl Task {
    pub fn new(id: u32, title: &str) -> Result<Self, TaskError> {
        let title = title.trim();
        if title.is_empty() {
            return Err(TaskError::EmptyTitle);
        }

        Ok(Task {
            id,
            title: title.to_string(),
            status: Status::Todo,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskStore {
    pub version: u32,
    pub next_id: u32,
    pub tasks: Vec<Task>,
}

impl TaskStore {
    pub fn add_task(&mut self, title: &str) -> Result<Task, TaskError> {
        let task = Task::new(self.next_id, title)?;

        self.tasks.push(task.clone());
        self.next_id += 1;

        Ok(task)
    }

    pub fn list_tasks(&self) -> &[Task] {
        &self.tasks
    }

    pub fn list_by_status(&self, status: Status) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|task| task.status == status)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Todo,
    InProgress,
    Done,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Todo => write!(f, "todo"),
            Status::InProgress => write!(f, "in-progress"),
            Status::Done => write!(f, "done"),
        }
    }
}

// TODO: Day 4 - parse args with std::env::args(), dispatch to command fns.
fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_empty_store() -> TaskStore {
        TaskStore {
            version: 1,
            next_id: 42,
            tasks: Vec::new(),
        }
    }

    #[test]
    fn status_display() {
        assert_eq!(Status::Todo.to_string(), "todo");
        assert_eq!(Status::InProgress.to_string(), "in-progress");
        assert_eq!(Status::Done.to_string(), "done");
    }

    #[test]
    fn status_display_matches_json() {
        for s in [Status::Todo, Status::InProgress, Status::Done] {
            assert_eq!(serde_json::to_string(&s).unwrap(), format!("\"{s}\""));
        }
    }

    #[test]
    fn task_display_format() {
        let dummy_task = Task {
            id: 1,
            title: String::from("Write a cool unit test"),
            status: Status::InProgress,
        };

        assert_eq!(
            dummy_task.to_string(),
            "[1] Write a cool unit test (in-progress)"
        );
    }

    #[test]
    fn taskstore_json_roundtrip() {
        let dummy_task = Task {
            id: 1,
            title: String::from("Test something..."),
            status: Status::Todo,
        };

        let store = TaskStore {
            version: 1,
            next_id: 2,
            tasks: vec![dummy_task],
        };
        let json = serde_json::to_string(&store).unwrap();
        assert_eq!(serde_json::from_str::<TaskStore>(&json).unwrap(), store);
    }

    #[test]
    fn taskstore_parses_documented_shape() {
        let json = r#"{"version":1,"next_id":2,
        "tasks":[{"id":1,"title":"Write README","status":"done"}]}"#;
        let store: TaskStore = serde_json::from_str(json).unwrap();
        assert_eq!(store.next_id, 2);
        assert_eq!(store.tasks[0].status, Status::Done);
    }

    #[test]
    fn error_display_version_too_new() {
        let err = TaskError::FileVersionTooNew { found: 2 };
        assert_eq!(
            err.to_string(),
            "task file was written by a newer version of this app (file version 2); update the app to open it"
        );
    }

    #[test]
    fn error_display_version_invalid() {
        let err = TaskError::FileVersionInvalid { found: 0 };
        assert_eq!(
            err.to_string(),
            "task file is corrupt or hand-edited (invalid version 0)"
        );
    }

    #[test]
    fn task_error_is_std_error() {
        let boxed: Box<dyn std::error::Error> =
            Box::new(TaskError::FileVersionInvalid { found: 0 });
        assert!(boxed.is::<TaskError>());
    }

    #[test]
    fn empty_title_display() {
        let err = TaskError::EmptyTitle;
        assert_eq!(err.to_string(), "task title cannot be empty");
    }

    #[test]
    fn task_creation_fails_with_empty_title() {
        let task_without_title = Task::new(1, "");

        assert!(task_without_title.is_err());
        assert!(matches!(
            task_without_title.unwrap_err(),
            TaskError::EmptyTitle
        ));
    }

    #[test]
    fn task_creation_succeeds_with_valid_title() {
        let task_with_title = Task::new(1, "Title");
        assert!(task_with_title.is_ok());
    }

    #[test]
    fn task_creation_fails_with_whitespace_title() {
        let task_with_whitespace_title = Task::new(1, "      ");

        assert!(task_with_whitespace_title.is_err());
        assert!(matches!(
            task_with_whitespace_title.unwrap_err(),
            TaskError::EmptyTitle
        ));
    }

    #[test]
    fn add_task_stores_task_and_allocates_starting_id() {
        let mut store = setup_empty_store();
        let starting_id = store.next_id;

        let result = store.add_task("Task 1");
        assert!(result.is_ok());

        let task = result.unwrap();
        assert_eq!(task.id, starting_id);
        assert_eq!(store.next_id, starting_id + 1);

        assert_eq!(store.tasks, vec![task]);
    }

    #[test]
    fn add_task_twice_allocates_distinct_incrementing_ids() {
        let mut store = setup_empty_store();

        let task1 = store.add_task("Task 1").unwrap();
        let task2 = store.add_task("Task 2").unwrap();

        assert_eq!(task1.id, 42);
        assert_eq!(task2.id, 43);
        assert!(task1.id < task2.id);
    }

    #[test]
    fn add_task_empty_title_guards_invariant_leaving_store_unmutated() {
        let mut store = setup_empty_store();
        let initial_id = store.next_id;

        let result = store.add_task("");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TaskError::EmptyTitle));

        assert_eq!(
            store.next_id, initial_id,
            "Invariant violated: next_id modified on failure."
        );
        assert_eq!(
            store.tasks.len(),
            0,
            "Invariant violated: task recorded on failure."
        );
    }

    #[test]
    fn list_tasks_on_empty_store_returns_empty_slice() {
        let store = setup_empty_store();
        let slice = store.list_tasks();

        assert!(slice.is_empty());
        assert_eq!(slice.len(), 0);
    }

    #[test]
    fn list_by_status_filters_correctly_and_handles_empty_matches() {
        let mut store = setup_empty_store();

        let t1 = store.add_task("Task 1").unwrap();
        let t2 = store.add_task("Task 2").unwrap();
        let t3 = store.add_task("Task 3").unwrap();

        store
            .tasks
            .iter_mut()
            .find(|t| t.id == t1.id)
            .expect("t1 must be in the store")
            .status = Status::InProgress;

        let mut t1_expected = t1;
        t1_expected.status = Status::InProgress;

        let todo_tasks = store.list_by_status(Status::Todo);
        assert_eq!(todo_tasks.len(), 2);
        assert_eq!(todo_tasks[0].id, t2.id);
        assert_eq!(todo_tasks[1].id, t3.id);

        let in_progress_tasks = store.list_by_status(Status::InProgress);
        assert_eq!(in_progress_tasks.len(), 1);
        assert_eq!(in_progress_tasks[0], &t1_expected);

        let done_tasks = store.list_by_status(Status::Done);
        assert!(done_tasks.is_empty());
        assert_eq!(done_tasks.len(), 0);
    }

    #[test]
    fn task_new_trims_whitespace_and_stores_sanitized_title() {
        let result = Task::new(1, "   Buy milk            ");

        assert!(result.is_ok());
        let task = result.unwrap();

        assert_eq!(task.title, "Buy milk");
    }
}
