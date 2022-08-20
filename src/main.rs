mod employee;
mod task;

use actix::{SyncArbiter};
use employee::{EmployeeCharacteristics, EmployeeResources};

use crate::{
    employee::{Employee, EmployeeType},
    task::Task,
};

static TICK_RATE: f32 = 10.;

fn main() {
    let system = actix::System::new();

    let addr = SyncArbiter::start(
        2, 
        || Employee::new(EmployeeType::Developer)
                    .with_characteristics(EmployeeCharacteristics::new())
                    .with_resources(EmployeeResources::new())
    );
    let addr_2 = SyncArbiter::start(
        2, 
        || Employee::new(EmployeeType::Developer)
                    .with_characteristics(EmployeeCharacteristics::new())
                    .with_resources(EmployeeResources {
                        energy: 50.0,
                        focus: 80.0,
                        stress: 10.0,
                    })
    );

    addr.do_send(Task { label: 1, ..Default::default() });
    addr.do_send(Task { label: 2, ..Default::default() });
    addr.do_send(Task { label: 3, ..Default::default() });
    addr_2.do_send(Task { label: 4, ..Default::default() });
    addr_2.do_send(Task { label: 5, ..Default::default() });

    system.run().expect("Something went wrong starting system.");
}
