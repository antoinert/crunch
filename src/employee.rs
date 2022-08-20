use std::time::Duration;

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
    pub employee_type: EmployeeType,
    pub characteristics: EmployeeCharacteristics,
    pub resources: EmployeeResources,
}

impl Employee {
    pub fn new(employee_type: EmployeeType) -> Employee {
        Employee {
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

    fn handle(&mut self, msg: Task, _ctx: &mut SyncContext<Self>) -> Self::Result {
        println!("Started task! {}", msg.label);

        let mut tasks = vec![Task::default()];

        loop {
            let mut to_remove = vec![];
            for (index, task) in tasks.iter_mut().enumerate() {
                task.process_tick(self);
                if task.is_done() {
                    println!("Finished {:?}", task.id);
                    to_remove.push(index);
                }
            }
            for id in to_remove {
                tasks.remove(id);
            }

            if tasks.len() == 0 {
                break;
            }
    
            sleep(Duration::from_secs_f32(1. / TICK_RATE));
        };

        println!("Finished task! {}", msg.label);
    }
}
