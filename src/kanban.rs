use std::collections::HashMap;

use actix::{Actor, Context, Handler, AsyncContext};

use crate::task::Task;

pub struct Kanban {
    task_list: HashMap<usize, Task>
}

impl Kanban {
    pub fn new() -> Self {
        Kanban { task_list: HashMap::new() }
    }
}

impl Actor for Kanban {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(10);

        ctx.notify(Task { ..Default::default() })
    }
}

impl Handler<Task> for Kanban {
    type Result = ();

    fn handle(&mut self, task: Task, _ctx: &mut Context<Self>) -> Self::Result {
        let task_list_size = *self.task_list.keys().last().unwrap_or(&1);
        self.task_list.insert(task_list_size, task);
        println!("Added task {:?} to task list. Current tasks: {:?}", task, self.task_list);
    }
}