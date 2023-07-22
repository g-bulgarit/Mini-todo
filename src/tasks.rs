use serde::Serialize;
use serde_json::Result;

use std::fs::File;
use std::io::Write;

#[derive(Clone, Copy, Serialize)]
pub enum TaskStatus {
    Backlog,
    InProgress,
    Done,
}

#[derive(Clone, Serialize)]
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

fn task_list_to_json_str(task_list: Vec<Task>) -> Result<String> {
    serde_json::to_string(&task_list)
}

pub fn save_tasks_to_file(
    backlog_list: Vec<Task>,
    inprogress_list: Vec<Task>,
    done_list: Vec<Task>,
) {
    let file_path = "saved_tasks.json";
    let mut file = File::create(file_path).unwrap();

    let backlog = task_list_to_json_str(backlog_list);
    let inprogress = task_list_to_json_str(inprogress_list);
    let done = task_list_to_json_str(done_list);

    file.write_all(backlog.expect("Should be able to read").as_bytes())
        .unwrap();
    file.write_all(inprogress.expect("Should be able to read").as_bytes())
        .unwrap();
    file.write_all(done.expect("Should be able to read").as_bytes())
        .unwrap();
}
