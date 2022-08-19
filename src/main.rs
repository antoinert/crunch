use std::{thread::sleep, time::Duration};

static TICK_RATE: f32 = 10.;

struct Employee {
    pub energy: f32,
    pub focus: f32
}

enum TaskType {
    CreatePR,
    CoffeeBreak
}

struct Task {
    pub id: TaskType,
    pub energy_cost_per_tick: f32,
    pub energy_provided: f32,
}

fn main() {
    let mut actor = Employee { energy: 30., focus: 30. };
    let mut create_pr_task = Task { id: TaskType::CreatePR, energy_cost_per_tick: 1., energy_provided: 0. };

    loop {


        cost_function(&mut actor, &mut create_pr_task);
        sleep(Duration::from_secs_f32(1. / TICK_RATE));
    }
}

fn cost_function(mut employee: &mut Employee, mut task: &mut Task) {
    let diff = match task.id {
        TaskType::CreatePR => {
            let total_cost= task.energy_cost_per_tick / employee.focus;

            employee.energy -= total_cost;
            task.energy_provided += total_cost;

            total_cost
        },
        TaskType::CoffeeBreak => {
            let total_cost = task.energy_cost_per_tick;

            employee.energy -= total_cost;
            task.energy_provided += total_cost;

            total_cost
        }
    };

    println!("{} energy used. Energy remaining: {}", diff, employee.energy);
}