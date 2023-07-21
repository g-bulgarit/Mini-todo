#[derive(Clone, Copy)]
pub enum TaskStatus {
    Backlog,
    InProgress,
    Done,
}

#[derive(Clone)]
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
