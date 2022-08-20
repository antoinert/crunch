mod employee;
mod task;
mod kanban;

use actix::{SyncArbiter, Actor};
use employee::{EmployeeCharacteristics, EmployeeResources};

use crate::{
    employee::{Employee, EmployeeType},
    task::Task,
    kanban::Kanban
};

static TICK_RATE: f32 = 10.;

fn main() {
    let system = actix::System::new();

    system.block_on(async { 
        let addr = SyncArbiter::start(
            1, 
            || Employee::new(EmployeeType::Developer, "John".to_string())
                        .with_characteristics(EmployeeCharacteristics::new())
                        .with_resources(EmployeeResources::new())
        );
        let addr_2 = SyncArbiter::start(
            1, 
            || Employee::new(EmployeeType::Developer, "Kelly".to_string())
                        .with_characteristics(EmployeeCharacteristics::new())
                        .with_resources(EmployeeResources {
                            energy: 50.0,
                            focus: 80.0,
                            stress: 10.0,
                        })
        );

        let kanban = Kanban::new().start();
        
        addr.do_send(Task { ..Default::default() });
        addr.do_send(Task { ..Default::default() });
        addr.do_send(Task { ..Default::default() });
        addr_2.do_send(Task { ..Default::default() });
        addr_2.do_send(Task { ..Default::default() });
    });

    system.run().expect("Something went wrong starting system.");
}
