use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering}, time::Duration,
};

use actix::{Actor, Context, Handler, Addr, Message, AsyncContext};

use crate::{task::{Task, WorkCompleted, Work}, employee::EmployeeActor, TICK_RATE};

pub fn create_task_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub struct Kanban {
    task_list: HashMap<usize, Task>,
    pub employee_addresses: Vec<Addr<EmployeeActor>>
}

impl Kanban {
    pub fn new() -> Self {
        Kanban {
            task_list: HashMap::new(),
            employee_addresses: vec![],
        }
    }

    pub fn tick(&self) {
        for (index, employee_address) in self.employee_addresses.iter().enumerate() {
            let mut undone_tasks = self.task_list.iter().filter(|(_, task)| !task.is_done());

            if let Some((j, task)) = undone_tasks.nth(index) {
                employee_address.do_send(Work {
                    task: task.id,
                    uuid: *j,
                })
            }
        }
    }
}

impl Actor for Kanban {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(10);

        ctx.run_interval(Duration::from_secs_f32(1. / TICK_RATE), |k, _| k.tick());
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
                "Work performed by {} {:?}, progress: {}%",
                work_completed.employee_name, task.id, task.energy_taken / task.total_energy_required * 100.
            );
        }
    }
}

pub struct AddEmployee {
    pub employee_address: Addr<EmployeeActor>
}

impl Message for AddEmployee {
    type Result = ();
}

impl Handler<AddEmployee> for Kanban {
    type Result = ();

    fn handle(&mut self, add_employee: AddEmployee, _ctx: &mut Context<Self>) -> Self::Result {
        self.employee_addresses.push(add_employee.employee_address);
    }
}