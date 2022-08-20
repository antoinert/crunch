use std::ops::AddAssign;

use actix::{Actor, Addr, Handler, Message, SyncArbiter, SyncContext};
use rand::Rng;

use crate::{
    task::{TaskId, Work, WorkCompleted},
    Kanban,
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

    pub fn empty() -> EmployeeResources {
        EmployeeResources {
            energy: 0.0,
            focus: 0.0,
            stress: 0.0,
        }
    }
}

impl AddAssign for EmployeeResources {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            energy: self.energy + other.energy,
            focus: self.focus + other.focus,
            stress: self.stress + other.stress,
        };
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

    fn spawn_tasks(&self) {
        let mut rng = rand::thread_rng();

        if self.resources.focus < 30. {
            if rng.gen_bool(0.01) {
                self.kanban_address.do_send(TaskId::CoffeeBreak.to_task());
                self.kanban_address.do_send(TaskId::CoffeeBreak.to_task())
            }
        }
    }
}

impl Actor for EmployeeActor {
    type Context = SyncContext<Self>;
}

impl Handler<Work> for EmployeeActor {
    type Result = ();

    fn handle(&mut self, work: Work, ctx: &mut SyncContext<Self>) -> Self::Result {
        self.spawn_tasks();

        let task_data = work.task.to_task();
        let multiplier = task_data.energy_multipliers.get_energy_cost(&self);
        let energy_add = task_data.energy_taken_per_tick * multiplier;

        self.resources.energy -= energy_add;
        self.resources.focus -= energy_add * 2.;

        self.kanban_address.do_send(WorkCompleted {
            employee_address: ctx.address(),
            employee_name: self.employee_name,
            uuid: work.uuid,
            energy_add,
        });
        // Send updated employee data
        self.kanban_address.do_send(self.clone());
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

pub enum BuffId {
    Caffeinated,
}

impl BuffId {
    fn translate_to_resources(&self) -> EmployeeResources {
        match *self {
            BuffId::Caffeinated => {
                let mut bonus_resources = EmployeeResources::empty();
                bonus_resources.focus += 30.0;
                bonus_resources.energy += 20.0;

                bonus_resources
            }
        }
    }
}

pub struct Buff {
    pub id: BuffId,
}

impl Message for Buff {
    type Result = ();
}

impl Handler<Buff> for EmployeeActor {
    type Result = ();

    fn handle(&mut self, buff: Buff, _ctx: &mut SyncContext<Self>) -> Self::Result {
        self.resources += buff.id.translate_to_resources();
    }
}

impl Message for EmployeeActor {
    type Result = ();
}
