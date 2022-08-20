use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

use actix::{Actor, AsyncContext, Context, Handler};

use crate::task::{Task, WorkCompleted};

pub fn create_task_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub struct Kanban {
    task_list: HashMap<usize, Task>,
}

impl Kanban {
    pub fn new() -> Self {
        Kanban {
            task_list: HashMap::new(),
        }
    }
}

impl Actor for Kanban {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(10);

        ctx.notify(Task {
            ..Default::default()
        })
    }
}

impl Handler<Task> for Kanban {
    type Result = ();

    fn handle(&mut self, task: Task, _ctx: &mut Context<Self>) -> Self::Result {
        self.task_list.insert(create_task_id(), task);
        println!(
            "Added task {:?} to task list. Current tasks: {:?}",
            task, self.task_list
        );
    }
}

impl Handler<WorkCompleted> for Kanban {
    type Result = ();

    fn handle(&mut self, work_completed: WorkCompleted, _ctx: &mut Context<Self>) -> Self::Result {
        if let Some(task) = self.task_list.get_mut(&work_completed.uuid) {
            task.energy_taken += work_completed.energy_add;
            println!(
                "Work performed {:?}, energy_add: {} / {}",
                task.id, task.energy_taken, task.total_energy_required
            );
        }
    }
}
