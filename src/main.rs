mod employee;
mod task;

use employee::{EmployeeCharacteristics, EmployeeResources};

use crate::{
    employee::{Employee, EmployeeType},
    task::Task,
};

#[allow(unused)]
static TICK_RATE: f32 = 10.;

fn main() {
    let system = actix::System::new();

    let employee1 = Employee::new(
        EmployeeType::Developer,
        "John",
        Default::default(),
        Default::default(),
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
    );

    employee1.addr.do_send(Task {
        ..Default::default()
    });
    employee1.addr.do_send(Task {
        ..Default::default()
    });
    employee1.addr.do_send(Task {
        ..Default::default()
    });
    employee2.addr.do_send(Task {
        ..Default::default()
    });
    employee2.addr.do_send(Task {
        ..Default::default()
    });

    system.run().expect("Something went wrong starting system.");
}
