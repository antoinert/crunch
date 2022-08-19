mod employee;
mod task;
use std::{thread::sleep, time::Duration};

use crate::{
    employee::{Employee, EmployeeType},
    task::Task,
};

static TICK_RATE: f32 = 10.;

fn main() {
    let mut actor = Employee::new(EmployeeType::Developer);
    let mut tasks = vec![Task::default()];

    loop {
        let mut to_remove = vec![];
        for (index, task) in tasks.iter_mut().enumerate() {
            task.process_tick(&mut actor);
            if task.is_done() {
                println!("Finished {:?}", task.id);
                to_remove.push(index);
            }
        }
        for id in to_remove {
            tasks.remove(id);
        }

        sleep(Duration::from_secs_f32(1. / TICK_RATE));
    }
}
