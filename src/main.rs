use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum TaskError {
    FileVersionTooNew { found: u32 },
    FileVersionInvalid { found: u32 },
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskStore {
    pub version: u32,
    pub next_id: u32,
    pub tasks: Vec<Task>,
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
}
