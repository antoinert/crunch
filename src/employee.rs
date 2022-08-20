use std::time::{Duration, Instant};

use rand::Rng;
use actix::{Actor, Context, System, Handler, AsyncContext, SyncContext};
use std::thread::sleep;
use crate::{Task, TICK_RATE};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum EmployeeType {
    Manager,
    Developer,
}

pub struct EmployeeCharacteristics {
    pub company_experience: f32,
    pub rigor: f32,
    pub programming_skills: f32,
    pub fitness: f32,
}

impl EmployeeCharacteristics {
    pub fn new() -> EmployeeCharacteristics {
        let mut rng = rand::thread_rng();
        EmployeeCharacteristics {
            company_experience: rng.gen_range(15.0..85.0),
            rigor: rng.gen_range(15.0..85.0),
            programming_skills: rng.gen_range(15.0..85.0),
            fitness: rng.gen_range(15.0..85.0),
        }
    }
}

pub struct EmployeeResources {
    pub energy: f32,
    pub focus: f32,
    pub stress: f32,
}

impl EmployeeResources {
    pub fn new() -> EmployeeResources {
        EmployeeResources {
            energy: 100.0,
            focus: 100.0,
            stress: 0.0,
        }
    }
}

pub struct Employee {
    pub employee_name: String,
    pub employee_type: EmployeeType,
    pub characteristics: EmployeeCharacteristics,
    pub resources: EmployeeResources,
}

impl Employee {
    pub fn new(employee_type: EmployeeType, name: String) -> Employee {
        Employee {
            employee_name: name,
            employee_type,
            characteristics: EmployeeCharacteristics::new(),
            resources: EmployeeResources::new(),
        }
    }

    pub fn with_characteristics(mut self, characteristics: EmployeeCharacteristics) -> Employee {
        self.characteristics = characteristics;

        self
    }

    pub fn with_resources(mut self, resources: EmployeeResources) -> Employee {
        self.resources = resources;

        self
    }
}

impl Actor for Employee {
    type Context = SyncContext<Self>;
}

impl Handler<Task> for Employee {
    type Result = ();

    fn handle(&mut self, mut task: Task, _ctx: &mut SyncContext<Self>) -> Self::Result {
        println!("{} started task {:?}!", self.employee_name, task.id);

        let timer = Instant::now();

        loop {
            task.process_tick(self);

            if task.is_done() {
                break;
            }

            sleep(Duration::from_secs_f32(1. / TICK_RATE));
        };

        println!("{} finished task {:?} in {} seconds!", self.employee_name, task.id, timer.elapsed().as_secs());
    }
}
