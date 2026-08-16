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
fn status_display_parse_roundtrip() {
    for s in [Status::Todo, Status::InProgress, Status::Done] {
        assert_eq!(s.to_string().parse::<Status>().unwrap(), s);
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
    let boxed: Box<dyn std::error::Error> = Box::new(TaskError::FileVersionInvalid { found: 0 });
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
#[test]
fn delete_task_removes_only_target_and_preserves_next_id() {
    let mut store = setup_empty_store();

    let t1 = store.add_task("Task 1").unwrap();
    let t2 = store.add_task("Task 2").unwrap();
    let t3 = store.add_task("Task 3").unwrap();

    let next_id_before = store.next_id;

    assert!(store.delete_task(t2.id).is_ok());

    assert_eq!(store.tasks.len(), 2);
    assert_eq!(store.tasks[0].id, t1.id);
    assert_eq!(store.tasks[1].id, t3.id);

    assert_eq!(
        store.next_id, next_id_before,
        "Invariant violated: next_id changed on delete; IDs must never be reused."
    );
}

#[test]
fn delete_task_unknown_id_errors_and_leaves_store_unmutated() {
    let mut store = setup_empty_store();

    let t1 = store.add_task("Task 1").unwrap();
    let next_id_before = store.next_id;

    let result = store.delete_task(99);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TaskError::TaskNotFound { id: 99 }
    ));

    assert_eq!(
        store.tasks.len(),
        1,
        "Invariant violated: task removed on failed delete."
    );
    assert_eq!(store.tasks[0].id, t1.id);
    assert_eq!(
        store.next_id, next_id_before,
        "Invariant violated: next_id modified on failure."
    );
}

#[test]
fn update_status_changes_only_target_task() {
    let mut store = setup_empty_store();

    let t1 = store.add_task("Task 1").unwrap();
    let t2 = store.add_task("Task 2").unwrap();

    assert!(store.update_status(t1.id, Status::Done).is_ok());

    assert_eq!(store.tasks[0].status, Status::Done);
    assert_eq!(
        store.tasks[1].status,
        Status::Todo,
        "update_status touched a task it was not given."
    );
    assert_eq!(store.tasks[1].id, t2.id);
}

#[test]
fn update_status_unknown_id_errors_and_leaves_statuses_untouched() {
    let mut store = setup_empty_store();

    store.add_task("Task 1").unwrap();
    store.add_task("Task 2").unwrap();

    let result = store.update_status(99, Status::Done);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TaskError::TaskNotFound { id: 99 }
    ));

    assert!(
        store.tasks.iter().all(|t| t.status == Status::Todo),
        "Invariant violated: a status changed on failed update."
    );
}

#[test]
fn status_from_str_is_lenient_about_case_and_whitespace() {
    assert_eq!("DONE".parse::<Status>().unwrap(), Status::Done);
    assert_eq!(
        " in-progress ".parse::<Status>().unwrap(),
        Status::InProgress
    );
    assert_eq!("ToDo".parse::<Status>().unwrap(), Status::Todo);
}

#[test]
fn status_from_str_rejects_unknown_and_echoes_original_input() {
    let err = " Finished ".parse::<Status>().unwrap_err();

    let TaskError::InvalidStatus { given } = err else {
        panic!("expected InvalidStatus, got {err:?}");
    };

    assert_eq!(
        given, " Finished ",
        "error must echo the caller's original input, not the normalised form."
    );
}

#[test]
fn error_display_unknown_command() {
    let err = TaskError::UnknownCommand {
        given: String::from("frobnicate"),
    };
    assert_eq!(
        err.to_string(),
        format!("unknown command \"frobnicate\"\n{USAGE}")
    );
}

#[test]
fn error_display_missing_command() {
    assert_eq!(TaskError::MissingCommand.to_string(), USAGE);
}

#[test]
fn error_display_unknown_argument() {
    let err = TaskError::UnknownArgument {
        given: String::from("--oops"),
    };
    assert_eq!(err.to_string(), "unknown argument \"--oops\"");
}

#[test]
fn error_display_missing_status_value() {
    assert_eq!(
        TaskError::MissingStatusValue.to_string(),
        "--status needs a value: expected todo, in-progress, or done"
    );
}
