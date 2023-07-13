enum TaskStatus {
    Backlog,
    InProgress,
    Done,
}

struct Task {
    status: TaskStatus,
    text: String,
}