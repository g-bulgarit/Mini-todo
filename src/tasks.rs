use serde::Serialize;
use serde_json::Result;

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

pub fn all_tasks_to_json(backlog_list: Vec<Task>, inprogress_list: Vec<Task>, done_list: Vec<Task>) -> Vec<String>{
    let backlog = task_list_to_json_str(backlog_list);
    let inprogress = task_list_to_json_str(inprogress_list);
    let done = task_list_to_json_str(done_list);
    let mut combined_list: Vec<String> = Vec::new();
    combined_list.push(backlog.unwrap());
    combined_list.push(inprogress.unwrap());
    combined_list.push(done.unwrap());
    combined_list
}