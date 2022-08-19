use crate::employee::EmployeeType;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TaskId {
    CreatePR,
    CoffeeBreak,
    PlanWeekTasks,
}

impl TaskId {
    pub fn to_task(&self) -> Task {
        match *self {
            TaskId::CreatePR => Task {
                employee_type: EmployeeType::Developer,
                total_energy_required: 10.0,
                energy_taken_per_tick: 0.1,
                ..Task::default()
            },
            TaskId::CoffeeBreak => Task {
                employee_type: EmployeeType::Developer,
                total_energy_required: 1.0,
                energy_taken_per_tick: 0.01,
                ..Task::default()
            },
            TaskId::PlanWeekTasks => Task {
                employee_type: EmployeeType::Manager,
                total_energy_required: 3.0,
                energy_taken_per_tick: 0.1,
                ..Task::default()
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: TaskId,
    pub employee_type: EmployeeType,
    /// How much energy is needed in total
    pub total_energy_required: f32,
    /// How much have been given already
    pub energy_taken: f32,
    /// How much energy will be given by actor at each tick
    pub energy_taken_per_tick: f32,
    pub consequences: Vec<TaskId>,
}

impl Task {
    fn done(&self) -> bool {
        self.total_energy_required <= self.energy_taken
    }
}

impl Default for Task {
    fn default() -> Self {
        Task {
            id: TaskId::CreatePR,
            employee_type: EmployeeType::Developer,
            total_energy_required: 5.0,
            energy_taken: 0.0,
            energy_taken_per_tick: 0.1,
            consequences: Vec::new(),
        }
    }
}
