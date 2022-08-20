use actix::Message;

use crate::employee::EmployeeActor;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TaskId {
    CreatePR,
    ReviewPR,
    MergePR,
    CoffeeBreak
}

impl TaskId {
    #[allow(unused)]
    pub fn to_task(&self) -> Task {
        match *self {
            TaskId::CreatePR => Task {
                total_energy_required: 10.0,
                energy_taken_per_tick: 0.1,
                energy_multipliers: TaskEnergyMultipliers {
                    company_experience: 2.0,
                    rigor: 2.0,
                    programming_skills: 2.0,
                    fitness: 1.0,
                    energy: 1.2,
                    focus: 1.5,
                    stress: 0.5,
                },
                ..Task::default()
            },
            TaskId::ReviewPR => Task {
                total_energy_required: 10.0,
                energy_taken_per_tick: 0.1,
                energy_multipliers: TaskEnergyMultipliers {
                    company_experience: 2.0,
                    rigor: 2.0,
                    programming_skills: 2.0,
                    fitness: 1.0,
                    energy: 1.2,
                    focus: 1.5,
                    stress: 0.5,
                },
                ..Task::default()
            },
            TaskId::MergePR => Task {
                total_energy_required: 1.0,
                energy_taken_per_tick: 0.1,
                energy_multipliers: TaskEnergyMultipliers {
                    company_experience: 2.0,
                    rigor: 2.0,
                    programming_skills: 2.0,
                    fitness: 1.0,
                    energy: 1.2,
                    focus: 1.5,
                    stress: 0.5,
                },
                ..Task::default()
            },
            TaskId::CoffeeBreak => Task {
                total_energy_required: 1.0,
                energy_taken_per_tick: 0.1,
                energy_multipliers: TaskEnergyMultipliers {
                    company_experience: 2.0,
                    rigor: 2.0,
                    programming_skills: 2.0,
                    fitness: 1.0,
                    energy: 1.2,
                    focus: 1.5,
                    stress: 0.5,
                },
                ..Task::default()
            },
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TaskEnergyMultipliers {
    // Characteristics
    pub company_experience: f32,
    pub rigor: f32,
    pub programming_skills: f32,
    pub fitness: f32,
    // Resources
    pub energy: f32,
    pub focus: f32,
    pub stress: f32,
}

impl TaskEnergyMultipliers {
    pub fn get_energy_cost(&self, employee: &EmployeeActor) -> f32 {
        1.0 * employee.characteristics.rigor / 100.
    }
}

impl Default for TaskEnergyMultipliers {
    fn default() -> Self {
        TaskEnergyMultipliers {
            company_experience: 1.0,
            rigor: 1.0,
            programming_skills: 1.0,
            fitness: 1.0,
            energy: 1.0,
            focus: 1.0,
            stress: 1.0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Task {
    pub id: TaskId,
    /// How much energy is needed in total
    pub total_energy_required: f32,
    /// How much have been given already
    pub energy_taken: f32,
    /// How much energy will be given by actor at each tick
    pub energy_taken_per_tick: f32,
    pub energy_multipliers: TaskEnergyMultipliers,
}

impl Task {
    #[allow(unused)]
    pub fn is_done(&self) -> bool {
        self.total_energy_required <= self.energy_taken
    }

    pub fn progress(&self) -> f32 {
        self.energy_taken / self.total_energy_required
    }
}

impl Default for Task {
    fn default() -> Self {
        Task {
            id: TaskId::CreatePR,
            total_energy_required: 5.0,
            energy_taken: 0.0,
            energy_taken_per_tick: 0.1,
            energy_multipliers: TaskEnergyMultipliers::default(),
        }
    }
}

impl Message for Task {
    type Result = ();
}

pub struct Work {
    pub task: TaskId,
    pub uuid: usize,
}

impl Message for Work {
    type Result = ();
}

pub struct WorkCompleted {
    pub employee_name: &'static str,
    pub uuid: usize,
    pub energy_add: f32,
}

impl Message for WorkCompleted {
    type Result = ();
}
