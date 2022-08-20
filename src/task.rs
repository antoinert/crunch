use actix::Message;

use crate::employee::{Employee, EmployeeCharacteristics, EmployeeResources, EmployeeType};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TaskId {
    CreatePR,
}

impl TaskId {
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
    fn get_energy_cost(
        &self,
        chars: &EmployeeCharacteristics,
        _resources: &EmployeeResources,
    ) -> f32 {
        1.0 * chars.rigor / 100.
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
    pub label: u32,
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
    pub fn is_done(&self) -> bool {
        self.total_energy_required <= self.energy_taken
    }

    pub fn process_tick(&mut self, employee: &mut Employee) {
        let multiplier = self
            .energy_multipliers
            .get_energy_cost(&employee.characteristics, &employee.resources);
        let energy_add = self.energy_taken_per_tick * multiplier;
        self.energy_taken += energy_add;
        // println!(
        //     "{:?} Energy taken {} out of {}. Tick: {}",
        //     self.id, self.energy_taken, self.total_energy_required, energy_add
        // );
    }
}

impl Default for Task {
    fn default() -> Self {
        Task {
            label: 1,
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