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
    pub energy_required: f32,
}

fn main() {
    let mut actor = Employee { energy: 30., focus: 30. };
    let mut create_pr_task = Task { id: TaskType::CreatePR, energy_cost_per_tick: 20., energy_provided: 0., energy_required: 30. };

    loop {
        if cost_function(&mut actor, &mut create_pr_task) <= 1. {
            reward_function(&mut actor, &mut create_pr_task);
        }
        sleep(Duration::from_secs_f32(1. / TICK_RATE));
    }
}

fn cost_function(mut employee: &mut Employee, mut task: &mut Task) -> f32 {
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

    if diff <= 0. {
        reward_function(employee, task);
    }

    println!("{} energy used. Energy remaining: {}", diff, employee.energy);
    task.energy_required / task.energy_provided
}

fn reward_function(mut employee: &mut Employee, mut task: &mut Task) {
    match task.id {
        TaskType::CreatePR => {
            println!("Finished PR, creating Merge task.");
        },
        TaskType::CoffeeBreak => {
            println!("Finished Coffee Break, granted Caffeinated.");
        }
    }
}