mod employee;
mod task;

use actix::{Actor, SyncArbiter};

use crate::{
    employee::{Employee, EmployeeType},
    task::Task,
};

static TICK_RATE: f32 = 10.;

fn main() {
    let system = actix::System::new();

    let addr = SyncArbiter::start(2, || Employee::new(EmployeeType::Developer));

    addr.do_send(Task::default());
    addr.do_send(Task::default());
    addr.do_send(Task::default());

    system.run().expect("Something went wrong starting system.");
}
