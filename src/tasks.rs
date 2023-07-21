use tui::widgets::ListItem;

#[derive(Clone, Copy)]
pub enum TaskStatus {
    Backlog,
    InProgress,
    Done,
}

#[derive(Clone)]
pub struct Task {
    status: TaskStatus,
    text: String,
}


impl Task {
    pub fn create_new_task(text: String, status: TaskStatus) -> Task{
        Task {status: status, text: text}
    }

    pub fn to_list_item(&self) -> ListItem<'static> {
        ListItem::new(self.text.clone())
    }

    pub fn get_status(&self) -> TaskStatus {
        self.status
    }
} 

pub fn by_name(name: String, tasks: Vec<Task>) -> Result<Task, u8> {
    for task in tasks {
        if task.text == name {
            return Ok(task);
        }
    }
    return Err(0)
}