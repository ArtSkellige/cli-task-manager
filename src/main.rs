use cli_task_manager::*;
use std::time::Duration;

fn extract_title(body: &str) -> Option<String> {
    let lower_body = body.to_ascii_lowercase();

    let open_tag_start = lower_body.find("<title")?;
    let remaining_lower = &lower_body[open_tag_start..];

    let open_tag_end_relative = remaining_lower.find('>')?;
    let content_start = open_tag_start + open_tag_end_relative + 1;

    let close_tag_start = lower_body[content_start..].find("</title>")?;
    let content_end = content_start + close_tag_start;

    let raw_title = body.get(content_start..content_end)?;

    let cleaned_title = raw_title
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");

    if cleaned_title.is_empty() {
        None
    } else {
        Some(cleaned_title)
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    let result = match args.get(1).map(String::as_str) {
        Some("add") => run_add(&args),
        Some("list") => run_list(&args),
        Some("delete") => run_delete(&args),
        Some("update") => run_update(&args),
        Some("fetch") => run_fetch(&args).await,
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

        store.save()?;
        println!("added: {task}");
        Ok(())
    }

    async fn run_fetch(args: &[String]) -> Result<(), TaskError> {
        let url = args.get(2).ok_or(TaskError::MissingUrl)?;

        if args.len() > 3 {
            return Err(TaskError::UnknownArgument {
                given: args[3].clone(),
            });
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(TaskError::FetchFailed)?;

        let body = client
            .get(url)
            .send()
            .await
            .map_err(TaskError::FetchFailed)?
            .text()
            .await
            .map_err(TaskError::FetchFailed)?;

        let title = extract_title(&body).unwrap_or_else(|| url.clone());

        let mut store = TaskStore::load()?;
        let task = store.add_task(&title)?;
        store.save()?;
        println!("fetched and added: {task}");
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
        store.save()?;
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
        store.save()?;
        println!("task {id} status updated to {new_status}.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::extract_title;

    #[test]
    fn extract_title_parses_plain_tag_with_mid_document_offsets() {
        let fixture = "<html><head><title>Hello</title></head></html>";
        assert_eq!(extract_title(fixture), Some("Hello".to_string()));
    }

    #[test]
    fn extract_title_skips_attributes_and_preserves_original_case() {
        let fixture = r#"<TITLE lang="en">GitHub Issue Tracker</TITLE>"#;
        assert_eq!(
            extract_title(fixture),
            Some("GitHub Issue Tracker".to_string())
        );
    }

    #[test]
    fn extract_title_collapses_interior_newlines_and_runs_of_whitespace() {
        let fixture = "<title>\n  My  \n  Multi-line  \t Task  \n</title>";
        assert_eq!(
            extract_title(fixture),
            Some("My Multi-line Task".to_string())
        );
    }

    #[test]
    fn extract_title_returns_none_for_structurally_empty_tag() {
        let fixture = "<title></title>";
        assert_eq!(extract_title(fixture), None);
    }

    #[test]
    fn extract_title_returns_none_when_tag_is_missing() {
        let fixture = "<html><body><h1>No title here</h1></body></html>";
        assert_eq!(extract_title(fixture), None);
    }

    #[test]
    fn extract_title_returns_none_for_whitespace_only_tag() {
        let fixture = "<title>   \n   \t  </title>";
        assert_eq!(extract_title(fixture), None);
    }
}
