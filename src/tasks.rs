use tui::widgets::Block;

enum TaskStatus {
    Backlog,
    InProgress,
    Done,
}

struct Task {
    status: TaskStatus,
    text: String,
}

impl From<Task> for Block {
    fn from(input: Task) -> Block{
        !todo()
    }

}