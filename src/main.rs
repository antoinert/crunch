mod employee;
mod kanban;
mod task;

use actix::Actor;
use kanban::AddEmployee;
use task::TaskId;

use crate::{
    employee::{Employee, EmployeeCharacteristics, EmployeeResources, EmployeeType},
    kanban::Kanban,
    task::Task,
};

fn main() {
    let system = actix::System::new();

    system.block_on(async {
        let kanban_address = Kanban::new().start();

        let employee1 = Employee::new(
            EmployeeType::Developer,
            "Okko",
            Default::default(),
            Default::default(),
            kanban_address.clone(),
        );
        let employee2 = Employee::new(
            EmployeeType::Developer,
            "Anton",
            EmployeeCharacteristics::new(),
            EmployeeResources {
                energy: 50.0,
                focus: 80.0,
                stress: 10.0,
            },
            kanban_address.clone(),
        );

        kanban_address.do_send(AddEmployee {
            employee_address: employee1.addr,
        });
        kanban_address.do_send(AddEmployee {
            employee_address: employee2.addr,
        });

        let feature_task = TaskId::CreatePR.to_task().as_feature();

        kanban_address.do_send(feature_task);
        kanban_address.do_send(feature_task);
    });

    system.run().expect("Something went wrong starting system.");
}
