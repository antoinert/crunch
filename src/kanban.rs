use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    io::{stdout, Stdout, Write},
    process::exit,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, System};
use crossterm::{
    cursor, event,
    event::{poll, Event, KeyCode, KeyEvent},
    queue, style,
    style::{Color, Stylize},
    terminal,
    terminal::enable_raw_mode,
    ExecutableCommand,
};
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
    task_list: HashMap<usize, (Task, BTreeSet<String>)>,
    done_list: VecDeque<(usize, TaskId, BTreeSet<String>)>,
    pub employee_addresses: Vec<Addr<EmployeeActor>>,
    employee_data: BTreeMap<String, EmployeeActor>,
    curr_employee: usize,
}

impl Kanban {
    pub fn new() -> Self {
        enable_raw_mode().unwrap();

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
            done_list: VecDeque::new(),
            employee_addresses: vec![],
            employee_data: BTreeMap::new(),
            curr_employee: 0,
        }
    }

    pub fn tick(&mut self, context: &mut Context<Kanban>) {
        self.handle_keys();

        let mut rng = rand::thread_rng();

        if rng.gen_bool(0.01) && self.task_list.len() < 10 {
            context.notify(Task::default());
        }

        for (index, employee_address) in self.employee_addresses.iter().enumerate() {
            if let Some((j, (task, _c))) = self.task_list.iter().nth(index) {
                employee_address.do_send(Work {
                    task: task.id,
                    uuid: *j,
                })
            }
        }
        self.draw()
    }

    fn handle_keys(&mut self) {
        if poll(Duration::from_millis(20)).unwrap() {
            let event = event::read();
            if let Ok(Event::Key(KeyEvent {
                code: KeyCode::Left,
                ..
            })) = event
            {
                if self.curr_employee == 0 {
                    self.curr_employee = self.employee_addresses.len() - 1;
                } else {
                    self.curr_employee -= 1;
                }
            }
            if let Ok(Event::Key(KeyEvent {
                code: KeyCode::Right,
                ..
            })) = event
            {
                self.curr_employee = (self.curr_employee + 1) % self.employee_addresses.len();
            }
            if let Ok(Event::Key(KeyEvent {
                code: KeyCode::Esc,
                ..
            })) = event
            {
                System::current().stop();
            }
        }
    }

    fn draw(&mut self) {
        let max_bar_width = 15;
        let progress_color = Color::Green;
        queue!(
            self.stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
        )
        .unwrap();
        let employees = self.employee_data.keys().cloned().collect::<Vec<String>>();
        if let Some(employee_name) = employees.get(self.curr_employee) {
            let curr_employee = self.employee_data.get(employee_name).unwrap();
            let mut employee_tasks = vec![];
            for (_id, (task, contributors)) in self.task_list.iter() {
                if contributors.contains(curr_employee.employee_name) {
                    employee_tasks.push(task.clone());
                }
            }
            draw_employee_card(&mut self.stdout, curr_employee, &employee_tasks);
        }

        // Title row
        queue!(self.stdout, style::Print("Tasks")).unwrap();

        for (id, (task, contributors)) in self.task_list.iter() {
            // Start row
            queue!(self.stdout, cursor::MoveToNextLine(1)).unwrap();
            // Title
            queue!(
                self.stdout,
                style::Print(&format!("{0: <20}", format!("{}: {:?}: ", *id, task.id)))
            );

            // Progress bar + percentage
            draw_task_progress(
                &mut self.stdout,
                progress_color,
                task.progress(),
                max_bar_width,
            );

            draw_contributors(&mut self.stdout, contributors);
        }
        queue!(self.stdout, cursor::MoveToNextLine(1), style::Print("Done"),).unwrap();
        // Draw done tasks
        for (uuid, task, contributors) in self.done_list.iter() {
            queue!(
                self.stdout,
                cursor::MoveToNextLine(1),
                style::Print(format!("{}: {:?} ", *uuid, *task))
            )
            .unwrap();

            draw_contributors(&mut self.stdout, contributors);
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
        self.task_list
            .insert(create_task_id(), (task, BTreeSet::new()));
    }
}

impl Handler<WorkCompleted> for Kanban {
    type Result = ();

    fn handle(&mut self, work_completed: WorkCompleted, ctx: &mut Context<Self>) -> Self::Result {
        if let Some((task, contributors)) = self.task_list.get_mut(&work_completed.uuid) {
            task.energy_taken += work_completed.energy_add;
            task.energy_taken = task.energy_taken.clamp(0.0, task.total_energy_required);
            contributors.insert(work_completed.employee_name.to_string());

            if task.is_done() {
                if let Some((task, contributors)) = self.task_list.remove(&work_completed.uuid) {
                    match task.id {
                        TaskId::CreatePR => ctx.notify(TaskId::ReviewPR.to_task()),
                        TaskId::MergePR => ctx.notify(TaskId::MergePR.to_task()),
                        _ => {}
                    }
                    self.done_list
                        .push_front((work_completed.uuid, task.id, contributors));
                    if self.done_list.len() > 10 {
                        self.done_list.pop_back();
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

impl Handler<EmployeeActor> for Kanban {
    type Result = ();

    fn handle(&mut self, employee_data: EmployeeActor, _ctx: &mut Context<Self>) -> Self::Result {
        self.employee_data
            .insert(employee_data.employee_name.to_string(), employee_data);
    }
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
        style::Print(format!(" {:.2} % ", progress * 100.0)),
    )
    .unwrap();
}

fn draw_contributors<W>(w: &mut W, contributors: &BTreeSet<String>)
where
    W: Write,
{
    let mut count = 0;
    for c in contributors.iter() {
        queue!(w, style::Print(format!("{c}"))).unwrap();
        count += 1;
        if count < contributors.len() {
            queue!(w, style::Print(", ")).unwrap();
        }
    }
}

fn draw_employee_card<W>(w: &mut W, employee: &EmployeeActor, employee_tasks: &Vec<Task>)
where
    W: Write,
{
    let card_height = 20;
    let card_width = 40;
    for y in 0..card_height {
        for x in 0..card_width {
            if (y == 0 || y == card_height - 1) || (x == 0 || x == card_width - 1) {
                queue!(
                    w,
                    cursor::MoveTo(x, y),
                    style::PrintStyledContent("█".underlined().green())
                )
                .unwrap();
            }
        }
    }
    queue!(
        w,
        cursor::MoveTo(1, 1),
        style::Print(format!("{}", employee.employee_name))
    )
    .unwrap();
    queue!(w, cursor::MoveTo(0, card_height + 1),).unwrap();
}
