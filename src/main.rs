use cli_task_manager::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let result = match args.get(1).map(String::as_str) {
        Some("add") => run_add(&args),
        Some("list") => run_list(&args),
        Some("delete") => run_delete(&args),
        Some("update") => run_update(&args),
        Some(unknown) => Err(TaskError::UnknownCommand {
            given: unknown.to_string(),
        }),
        None => Err(TaskError::MissingCommand),
    };

    if let Err(e) = result {
        match e {
            TaskError::MissingCommand => eprintln!("{USAGE}"),
            other => eprintln!("error: {other}"),
        }
        std::process::exit(1);
    }

    fn run_add(args: &[String]) -> Result<(), TaskError> {
        if args.len() < 3 {
            return Err(TaskError::EmptyTitle);
        }

        let title = args[2..].join(" ");

        let mut store = TaskStore::load()?;
        let task = store.add_task(&title)?;

        println!("added: {task}");
        Ok(())
    }

    fn run_list(args: &[String]) -> Result<(), TaskError> {
        if args.len() > 4 {
            return Err(TaskError::UnknownArgument {
                given: args[4].clone(),
            });
        }

        let store = TaskStore::load()?;

        let tasks: Vec<&Task> = match (
            args.get(2).map(String::as_str),
            args.get(3).map(String::as_str),
        ) {
            (None, _) => store.list_tasks().iter().collect(),
            (Some("--status"), Some(s)) => store.list_by_status(s.parse()?),
            (Some("--status"), None) => {
                return Err(TaskError::MissingStatusValue);
            }
            (Some(other), _) => {
                return Err(TaskError::UnknownArgument {
                    given: other.to_string(),
                });
            }
        };

        for task in tasks {
            println!("{task}");
        }
        Ok(())
    }

    fn run_delete(args: &[String]) -> Result<(), TaskError> {
        if args.len() > 3 {
            return Err(TaskError::UnknownArgument {
                given: args[3].clone(),
            });
        }

        let id_str = args.get(2).map(String::as_str).unwrap_or("");
        let id: u32 = id_str.parse().map_err(|_| TaskError::InvalidId {
            given: id_str.to_string(),
        })?;

        let mut store = TaskStore::load()?;

        store.delete_task(id)?;
        println!("task {id} deleted successfully.");
        Ok(())
    }

    fn run_update(args: &[String]) -> Result<(), TaskError> {
        if args.len() > 4 {
            return Err(TaskError::UnknownArgument {
                given: args[4].clone(),
            });
        }

        let id_str = args.get(2).map(String::as_str).unwrap_or("");
        let status_str = args.get(3).map(String::as_str).unwrap_or("");

        let id: u32 = id_str.parse().map_err(|_| TaskError::InvalidId {
            given: id_str.to_string(),
        })?;

        let new_status: Status = status_str.parse()?;

        let mut store = TaskStore::load()?;

        store.update_status(id, new_status)?;
        println!("task {id} status updated to {new_status}.");
        Ok(())
    }
}
