use std::{
    borrow::Borrow,
    thread::sleep,
    time::{Duration, Instant},
};

use actix::{Actor, Addr, AsyncContext, Context, Handler, SyncArbiter, SyncContext, System};
use rand::Rng;

use crate::{
    task::{Work, WorkCompleted},
    Kanban, Task, TICK_RATE,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum EmployeeType {
    #[allow(unused)]
    Manager,
    Developer,
}

#[derive(Debug, Copy, Clone)]
pub struct EmployeeCharacteristics {
    pub company_experience: f32,
    pub rigor: f32,
    pub programming_skills: f32,
    pub fitness: f32,
}

impl Default for EmployeeCharacteristics {
    fn default() -> Self {
        EmployeeCharacteristics::new()
    }
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

#[derive(Debug, Copy, Clone)]
pub struct EmployeeResources {
    pub energy: f32,
    pub focus: f32,
    pub stress: f32,
}

impl Default for EmployeeResources {
    fn default() -> Self {
        EmployeeResources::new()
    }
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

#[derive(Debug, Clone)]
pub struct EmployeeActor {
    pub employee_name: &'static str,
    pub employee_type: EmployeeType,
    pub characteristics: EmployeeCharacteristics,
    pub resources: EmployeeResources,
    pub kanban_address: Addr<Kanban>,
}

impl EmployeeActor {
    pub fn new(
        employee_type: EmployeeType,
        name: &'static str,
        characteristics: EmployeeCharacteristics,
        resources: EmployeeResources,
        kanban_address: Addr<Kanban>,
    ) -> EmployeeActor {
        EmployeeActor {
            employee_name: name,
            employee_type,
            characteristics,
            resources,
            kanban_address,
        }
    }
}

impl Actor for EmployeeActor {
    type Context = SyncContext<Self>;
}

impl Handler<Work> for EmployeeActor {
    type Result = ();

    fn handle(&mut self, mut work: Work, _ctx: &mut SyncContext<Self>) -> Self::Result {
        println!("Tick task {:?} by {}!", work.task, self.employee_name);

        let task_data = work.task.to_task();
        let multiplier = task_data.energy_multipliers.get_energy_cost(&self);
        let energy_add = task_data.energy_taken_per_tick * multiplier;
        self.kanban_address.do_send(WorkCompleted {
            uuid: work.uuid,
            energy_add,
        });
        // ToDo: Send energy add to right task in kanban
    }
}

pub struct Employee {
    pub addr: Addr<EmployeeActor>,
}

impl Employee {
    pub fn new(
        employee_type: EmployeeType,
        name: &'static str,
        characteristics: EmployeeCharacteristics,
        resources: EmployeeResources,
        kanban_address: Addr<Kanban>,
    ) -> Employee {
        Employee {
            addr: SyncArbiter::start(1, move || {
                EmployeeActor::new(
                    employee_type,
                    name,
                    characteristics,
                    resources,
                    kanban_address.clone(),
                )
            }),
        }
    }
}
