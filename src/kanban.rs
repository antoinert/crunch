use std::{
    collections::HashMap,
    io::{stdout, Stdout, Write},
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use crossterm::{cursor, queue, style, style::Color, terminal, ExecutableCommand};
use rand::Rng;

use crate::{
    employee::EmployeeActor,
    task::{Task, TaskId, Work, WorkCompleted},
};

static TICK_RATE: f32 = 10.;

pub fn create_task_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub struct Kanban {
    stdout: Stdout,
    task_list: HashMap<usize, Task>,
    pub employee_addresses: Vec<Addr<EmployeeActor>>,
}

impl Kanban {
    pub fn new() -> Self {
        let mut stdout = stdout();
        stdout
            .execute(terminal::Clear(terminal::ClearType::All))
            .unwrap();

        // Reset terminal
        queue!(
            stdout,
            terminal::EnterAlternateScreen,
            style::ResetColor,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(1, 1),
            cursor::Show,
            cursor::EnableBlinking
        )
        .unwrap();

        stdout.flush().unwrap();

        Kanban {
            stdout,
            task_list: HashMap::new(),
            employee_addresses: vec![],
        }
    }

    pub fn tick(&mut self, context: &mut Context<Kanban>) {
        let mut rng = rand::thread_rng();

        if rng.gen_bool(0.01) && self.task_list.len() < 10 {
            context.notify(Task::default());
        }

        for (index, employee_address) in self.employee_addresses.iter().enumerate() {
            if let Some((j, task)) = self.task_list.iter().nth(index) {
                employee_address.do_send(Work {
                    task: task.id,
                    uuid: *j,
                })
            }
        }
        self.draw()
    }

    fn draw(&mut self) {
        let title_width = 12;
        let max_bar_width = 15;
        let mut row = 0;
        let progress_color = Color::Green;
        // Title row
        queue!(self.stdout, cursor::MoveTo(0, 0), style::Print("Tasks")).unwrap();

        for (id, task) in self.task_list.iter() {
            // Start row
            queue!(self.stdout, cursor::MoveTo(0, row + 1),).unwrap();
            // Title
            draw_task_title(
                &mut self.stdout,
                &format!("{}: {:?}", *id, task.id),
                title_width,
            );
            // Progress bar + percentage
            draw_task_progress(
                &mut self.stdout,
                progress_color,
                task.progress(),
                max_bar_width,
            );
            row += 1;
        }
        // Flush last
        self.stdout.flush().unwrap();
    }
}

impl Drop for Kanban {
    fn drop(&mut self) {
        queue!(self.stdout, terminal::LeaveAlternateScreen).unwrap();
        self.stdout.flush().unwrap();
    }
}

impl Actor for Kanban {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(10);

        ctx.run_interval(
            Duration::from_secs_f32(1. / TICK_RATE),
            |kanban, context| kanban.tick(context),
        );
    }
}

impl Handler<Task> for Kanban {
    type Result = ();

    fn handle(&mut self, task: Task, _ctx: &mut Context<Self>) -> Self::Result {
        self.task_list.insert(create_task_id(), task);
    }
}

impl Handler<WorkCompleted> for Kanban {
    type Result = ();

    fn handle(&mut self, work_completed: WorkCompleted, ctx: &mut Context<Self>) -> Self::Result {
        if let Some(task) = self.task_list.get_mut(&work_completed.uuid) {
            task.energy_taken += work_completed.energy_add;
            task.energy_taken = task.energy_taken.clamp(0.0, task.total_energy_required);

            if task.is_done() {
                if let Some(task) = self.task_list.remove(&work_completed.uuid) {
                    match task.id {
                        TaskId::CreatePR => ctx.notify(TaskId::ReviewPR.to_task()),
                        _ => {}
                    }
                }
            }
        }
    }
}

pub struct AddEmployee {
    pub employee_address: Addr<EmployeeActor>,
}

impl Message for AddEmployee {
    type Result = ();
}

impl Handler<AddEmployee> for Kanban {
    type Result = ();

    fn handle(&mut self, add_employee: AddEmployee, _ctx: &mut Context<Self>) -> Self::Result {
        self.employee_addresses.push(add_employee.employee_address);
    }
}

fn draw_task_title<W>(w: &mut W, title: &str, max_title_length: usize)
where
    W: Write,
{
    queue!(
        w,
        style::Print(format!("{:.len$}: ", title, len = max_title_length))
    )
    .unwrap();
}

fn draw_task_progress<W>(w: &mut W, color: Color, progress: f32, max_width: u16)
where
    W: Write,
{
    let limit = (progress * max_width as f32).round() as u16;
    for col in 0..=max_width {
        if col <= limit {
            queue!(w, style::SetForegroundColor(color), style::Print("███")).unwrap();
        } else {
            queue!(
                w,
                style::SetForegroundColor(Color::Black),
                style::Print("███")
            )
            .unwrap();
        }
    }
    queue!(
        w,
        style::SetForegroundColor(Color::White),
        style::Print(format!(" {:.2} %", progress * 100.0)),
    )
    .unwrap();
}
