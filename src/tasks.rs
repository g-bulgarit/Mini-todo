use serde::{Serialize, Deserialize};
use serde_json::{Result, Value};

use std::fs::File;
use std::io::{Write, Read};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum TaskStatus {
    Backlog,
    InProgress,
    Done,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Task {
    pub status: TaskStatus,
    pub text: String,
}

impl Task {
    pub fn create_new_task(text: String, status: TaskStatus) -> Task {
        Task {
            status: status,
            text: text,
        }
    }

    pub fn change_status(&mut self, status: TaskStatus) {
        self.status = status;
    }
}

fn task_list_to_json_str(json_content: Value) -> Result<String> {
    serde_json::to_string_pretty(&json_content)
}

fn task_list_from_json_content(json_content: Value) -> Vec<Task> {
    todo!()
}

pub fn save_tasks_to_file(
    backlog_list: Vec<Task>,
    inprogress_list: Vec<Task>,
    done_list: Vec<Task>,
) {
    let file_path = "saved_tasks.json";
    let mut file = File::create(file_path).unwrap();

    let json_data = serde_json::json!({
        "tasks": [backlog_list, inprogress_list, done_list]
        }
    );

    let full_result = task_list_to_json_str(json_data);

    file.write_all(full_result.expect("Should be able to read").as_bytes())
        .unwrap();
}

pub fn read_tasks_from_file() -> Result<(Vec<Task>, Vec<Task>, Vec<Task>)> {
    let file_path = "saved_tasks.json";
    let mut file = File::open(file_path).unwrap();
    let mut json_file_content = String::new();
    file.read_to_string(&mut json_file_content).expect("File must exist");
    let parsed_json_file: Value = serde_json::from_str(&json_file_content).unwrap();

    let backlog_items = parsed_json_file["tasks"][0].as_array().unwrap().to_owned();
    let in_progress_items = parsed_json_file["tasks"][1].as_array().unwrap().to_owned();
    let done_items = parsed_json_file["tasks"][2].as_array().unwrap().to_owned();

    let backlog_list = backlog_items.iter()
        .filter_map(|task| serde_json::from_value(task.clone()).ok())
        .collect();

    let in_progress_list = in_progress_items.iter()
        .filter_map(|task| serde_json::from_value(task.clone()).ok())
        .collect();

    let done_list = done_items.iter()
        .filter_map(|task| serde_json::from_value(task.clone()).ok())
        .collect();

    Ok((backlog_list, in_progress_list, done_list))

}
