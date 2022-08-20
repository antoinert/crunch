mod employee;
mod kanban;
mod task;

use actix::{Actor, SyncArbiter};

use crate::{
    employee::{Employee, EmployeeCharacteristics, EmployeeResources, EmployeeType},
    kanban::Kanban,
    task::{Task, TaskId, Work},
};

#[allow(unused)]
static TICK_RATE: f32 = 10.;

fn main() {
    let system = actix::System::new();

    system.block_on(async {
        let kanban = Kanban::new().start();

        let employee1 = Employee::new(
            EmployeeType::Developer,
            "John",
            Default::default(),
            Default::default(),
            kanban.clone(),
        );
        let employee2 = Employee::new(
            EmployeeType::Developer,
            "Kelly",
            EmployeeCharacteristics::new(),
            EmployeeResources {
                energy: 50.0,
                focus: 80.0,
                stress: 10.0,
            },
            kanban,
        );

        employee1.addr.do_send(Work {
            task: TaskId::CreatePR,
        });
        employee1.addr.do_send(Work {
            task: TaskId::CreatePR,
        });
        employee1.addr.do_send(Work {
            task: TaskId::CreatePR,
        });
        employee2.addr.do_send(Work {
            task: TaskId::CreatePR,
        });
        employee2.addr.do_send(Work {
            task: TaskId::CreatePR,
        });
    });

    system.run().expect("Something went wrong starting system.");
}
