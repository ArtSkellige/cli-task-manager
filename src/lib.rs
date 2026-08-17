use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::path::Path;
use std::str::FromStr;

pub const USAGE: &str = "usage: task-manager <add|list|delete|update> [args]";
const DATA_PATH: &str = "tasks.json";

#[derive(Debug)]
pub enum TaskError {
    FileVersionTooNew { found: u32 },
    FileVersionInvalid { found: u32 },
    EmptyTitle,
    TaskNotFound { id: u32 },
    InvalidStatus { given: String },
    InvalidId { given: String },
    UnknownArgument { given: String },
    UnknownCommand { given: String },
    MissingCommand,
    MissingStatusValue,
    ReadFailed(std::io::Error),
    WriteFailed(std::io::Error),
    Json(serde_json::Error),
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
            TaskError::TaskNotFound { id } => write!(f, "no task with id {id}"),
            TaskError::InvalidStatus { given } => write!(
                f,
                "unknown status \"{given}\": expected todo, in-progress, or done"
            ),
            TaskError::InvalidId { given } => {
                write!(f, "invalid task id \"{given}\": expected a number")
            }
            TaskError::UnknownArgument { given } => {
                write!(f, "unknown argument \"{given}\"")
            }
            TaskError::UnknownCommand { given } => {
                write!(f, "unknown command \"{given}\"\n{USAGE}")
            }
            TaskError::MissingCommand => write!(f, "{USAGE}"),
            TaskError::MissingStatusValue => write!(
                f,
                "--status needs a value: expected todo, in-progress, or done"
            ),
            TaskError::ReadFailed(e) => write!(f, "could not read ./{DATA_PATH}: {e}"),
            TaskError::WriteFailed(e) => write!(f, "could not write ./{DATA_PATH}: {e}"),
            TaskError::Json(e) => write!(f, "./{DATA_PATH} is not valid JSON: {e}"),
        }
    }
}

impl Error for TaskError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TaskError::ReadFailed(e) | TaskError::WriteFailed(e) => Some(e),
            TaskError::Json(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct TaskStore {
    pub version: u32,
    pub next_id: u32,
    pub tasks: Vec<Task>,
}

impl TaskStore {
    pub fn load() -> Result<TaskStore, TaskError> {
        Self::load_from(Path::new(DATA_PATH))
    }

    pub fn load_from(path: &Path) -> Result<TaskStore, TaskError> {
        let data = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(TaskStore {
                    version: 1,
                    next_id: 1,
                    tasks: Vec::new(),
                });
            }
            Err(e) => return Err(TaskError::ReadFailed(e)),
        };

        Self::from_json(&data)
    }

    fn from_json(data: &str) -> Result<TaskStore, TaskError> {
        let store: TaskStore = serde_json::from_str(data).map_err(TaskError::Json)?;

        match store.version {
            1 => Ok(store),
            v if v > 1 => Err(TaskError::FileVersionTooNew { found: v }),
            v => Err(TaskError::FileVersionInvalid { found: v }),
        }
    }

    pub fn save(&self) -> Result<(), TaskError> {
        self.save_to(Path::new(DATA_PATH))
    }

    pub fn save_to(&self, path: &Path) -> Result<(), TaskError> {
        let json = serde_json::to_string_pretty(self).map_err(TaskError::Json)?;
        let temp = path.with_extension("json.tmp");

        std::fs::write(&temp, &json).map_err(TaskError::WriteFailed)?;
        std::fs::rename(&temp, path).map_err(TaskError::WriteFailed)?;

        Ok(())
    }

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

    pub fn delete_task(&mut self, id: u32) -> Result<(), TaskError> {
        let before = self.tasks.len();

        self.tasks.retain(|t| t.id != id);

        if self.tasks.len() == before {
            return Err(TaskError::TaskNotFound { id });
        }

        Ok(())
    }

    pub fn update_status(&mut self, id: u32, new_status: Status) -> Result<(), TaskError> {
        match self.tasks.iter_mut().find(|t| t.id == id) {
            Some(task) => {
                task.status = new_status;
                Ok(())
            }
            None => Err(TaskError::TaskNotFound { id }),
        }
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

impl FromStr for Status {
    type Err = TaskError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "todo" => Ok(Status::Todo),
            "in-progress" => Ok(Status::InProgress),
            "done" => Ok(Status::Done),
            _ => Err(TaskError::InvalidStatus {
                given: s.to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests;
