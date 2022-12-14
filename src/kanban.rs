use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    io::{stdout, Stdout, Write},
    sync::atomic::AtomicUsize,
    time::Duration,
};

use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, System};
use chrono::{DateTime, Utc};
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
    employee::{Buff, BuffId, EmployeeActor},
    task::{Task, TaskId, Work, WorkCompleted},
};

static TICK_RATE: f32 = 10.;

pub fn create_task_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

pub struct Kanban {
    stdout: Stdout,
    task_list: HashMap<usize, (Task, BTreeSet<String>)>,
    done_list: VecDeque<(usize, Task, BTreeSet<String>)>,
    pub employee_addresses: Vec<Addr<EmployeeActor>>,
    employee_data: BTreeMap<String, EmployeeActor>,
    curr_employee: usize,
    start_time: DateTime<Utc>,
    okko: &'static str,
    anton: &'static str
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
        let time = Utc::now();
        Kanban {
            stdout,
            task_list: HashMap::new(),
            done_list: VecDeque::new(),
            employee_addresses: vec![],
            employee_data: BTreeMap::new(),
            curr_employee: 0,
            start_time: time,
            okko: include_str!("../okko.txt"),
            anton: include_str!("../anton.txt")
        }
    }

    pub fn tick(&mut self, context: &mut Context<Kanban>) {
        self.handle_keys();

        let mut rng = rand::thread_rng();

        if rng.gen_bool(0.01) && self.task_list.len() < 10 {
            let task = if rng.gen_bool(0.2) {
                TaskId::CreatePR.to_task().as_bug_fix()
            } else {
                TaskId::CreatePR.to_task().as_feature()
            };

            context.notify(task);
        }

        let mut task_list = self
            .task_list
            .clone()
            .into_iter()
            .map(|(uuid, val)| (uuid, val))
            .collect::<Vec<(usize, (Task, BTreeSet<String>))>>();
        task_list.sort_by(|a, b| {
            let task_a: Task = a.1 .0;
            let task_b: Task = b.1 .0;

            if task_b.id.priority() == task_a.id.priority() {
                b.1 .0.progress().partial_cmp(&a.1 .0.progress()).unwrap()
            } else {
                task_b.id.priority().cmp(&task_a.id.priority())
            }
        });

        for (index, employee_address) in self.employee_addresses.iter().enumerate() {
            if let Some((j, (task, _c))) = task_list.iter().nth(index) {
                employee_address.do_send(Work {
                    task: task.id,
                    uuid: *j,
                })
            }
        }
        self.draw(&task_list)
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
                queue!(
                    self.stdout,
                    terminal::LeaveAlternateScreen,
                    style::ResetColor,
                    terminal::Clear(terminal::ClearType::All),
                    cursor::MoveTo(0, 0),
                    cursor::Show,
                    cursor::EnableBlinking
                )
                .unwrap();
                self.stdout.flush().unwrap();
                System::current().stop();
            }
        }
    }

    fn draw(&mut self, sorted_task_list: &Vec<(usize, (Task, BTreeSet<String>))>) {
        let max_bar_width = 15;
        let progress_color = Color::Green;
        let done_color = Color::Blue;
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
            draw_employee_card(&mut self.stdout, curr_employee, &employee_tasks, &self.okko, &self.anton);
        }

        // List all employees
        let employess = BTreeSet::from_iter(employees);
        queue!(self.stdout, style::Print("Super Dev Organization: ".red()),).unwrap();
        draw_contributors(&mut self.stdout, &employess);
        queue!(self.stdout, cursor::MoveToNextLine(1),).unwrap();

        draw_time_bar(&mut self.stdout, self.start_time);

        // Title row
        queue!(
            self.stdout,
            style::Print("Tasks"),
            cursor::MoveToNextLine(1)
        )
        .unwrap();

        let capped_list = &sorted_task_list[0..6.min(sorted_task_list.len())];
        for (_id, (task, contributors)) in capped_list.iter() {
            // Start row
            queue!(self.stdout, cursor::MoveToNextLine(1)).unwrap();
            let print = if task.id == TaskId::CoffeeBreak {
                style::Print(format!(
                    "{0: <23}",
                    format!("??? [{:<11}]         ", format!("{:?}", task.id))
                ))
            } else {
                style::Print(format!(
                    "{0: <23}",
                    format!("??? [{:<11}] {} ", format!("{:?}", task.id), task.name)
                ))
            };

            // Title
            queue!(
                self.stdout,
                print
            )
            .unwrap();

            // Progress bar + percentage
            draw_task_progress(
                &mut self.stdout,
                progress_color,
                task.progress(),
                max_bar_width,
            );

            draw_contributors(&mut self.stdout, contributors);
        }
        if !self.done_list.is_empty() {
            queue!(
                self.stdout,
                cursor::MoveToNextLine(1),
                cursor::MoveToNextLine(1),
                style::Print("Done"),
                cursor::MoveToNextLine(1)
            )
            .unwrap();
        }

        // Draw done tasks
        for (_uuid, task, contributors) in self.done_list.iter() {
            let print = if task.id == TaskId::CoffeeBreak {
                style::Print(format!(
                    "{0: <23}",
                    format!("??? [{:<11}]         ", format!("{:?}", task.id))
                ))
            } else {
                style::Print(format!(
                    "{0: <23}",
                    format!("??? [{:<11}] {} ", format!("{:?}", task.id), task.name)
                ))
            };

            queue!(
                self.stdout,
                cursor::MoveToNextLine(1),
                print
            )
            .unwrap();
            draw_task_progress(&mut self.stdout, done_color, 1.0, max_bar_width);

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
                        TaskId::ReviewPR => ctx.notify(TaskId::MergePR.to_task()),
                        TaskId::CoffeeBreak => {
                            if contributors.len() > 1 {
                                self.employee_addresses.iter().for_each(|addr| {
                                    addr.do_send(Buff {
                                        id: BuffId::Caffeinated,
                                    })
                                })
                            } else {
                                work_completed.employee_address.do_send(Buff {
                                    id: BuffId::Caffeinated,
                                });
                            }
                        }
                        _ => {}
                    }
                    self.done_list
                        .push_front((work_completed.uuid, task, contributors));
                    if self.done_list.len() > 5 {
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
        if col < limit || progress == 1.0 {
            queue!(w, style::SetForegroundColor(color), style::Print("?????????")).unwrap();
        } else {
            queue!(
                w,
                style::SetForegroundColor(Color::Black),
                style::Print("?????????")
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

fn draw_employee_card<W>(w: &mut W, employee: &EmployeeActor, employee_tasks: &Vec<Task>, okko: &str, anton: &str)
where
    W: Write,
{
    let card_height = 20;
    let card_width = 90;

    for y in 0..card_height {
        for x in 0..card_width {
            if (y == 0 || y == card_height - 1) || (x == 0 || x == card_width - 1) {
                queue!(
                    w,
                    cursor::MoveTo(x, y),
                    style::PrintStyledContent("???".underlined().green())
                )
                .unwrap();
            }
        }
    }

    let name_file = if employee.employee_name == "Okko" {
        "okko.txt".to_string()
    } else {
        "anton.txt".to_string()
    };

    queue!(w, cursor::MoveTo(1, 1),).unwrap();

    draw_file_name(w, name_file, okko, anton);

    draw_current_tasks(w, employee_tasks);

    draw_characteristics(w, employee);

    draw_resources(w, employee);

    queue!(w, cursor::MoveTo(0, card_height + 1),).unwrap();
}

fn draw_characteristics<W>(w: &mut W, employee: &EmployeeActor)
where
    W: Write,
{
    let section_start = (69, 2);

    queue!(
        w,
        cursor::MoveTo(section_start.0, section_start.1),
        style::PrintStyledContent("Characteristics".underlined().red()),
        cursor::MoveTo(section_start.0, section_start.1 + 2),
        style::Print(&format!(
            "{0: <20}",
            format!("Rigor: {:.0}", employee.characteristics.rigor)
        )),
        cursor::MoveTo(section_start.0, section_start.1 + 3),
        style::Print(&format!(
            "{0: <20}",
            format!(
                "Experience: {:.0}",
                employee.characteristics.company_experience
            )
        )),
        cursor::MoveTo(section_start.0, section_start.1 + 4),
        style::Print(&format!(
            "{0: <20}",
            format!("Skills: {:.0}", employee.characteristics.programming_skills)
        )),
        cursor::MoveTo(section_start.0, section_start.1 + 5),
        style::Print(&format!(
            "{0: <20}",
            format!("Fitness: {:.0}", employee.characteristics.fitness)
        )),
    )
    .unwrap();
}

fn draw_resources<W>(w: &mut W, employee: &EmployeeActor)
where
    W: Write,
{
    let section_start = (69, 9);

    queue!(
        w,
        cursor::MoveTo(section_start.0, section_start.1),
        style::PrintStyledContent("Resources".underlined().red()),
        cursor::MoveTo(section_start.0, section_start.1 + 2),
        style::Print(&format!(
            "{0: <20}",
            format!("Energy: {:.0}", employee.resources.energy)
        )),
        cursor::MoveTo(section_start.0, section_start.1 + 3),
        style::Print(&format!(
            "{0: <20}",
            format!("Focus: {:.0}", employee.resources.focus)
        )),
        cursor::MoveTo(section_start.0, section_start.1 + 4),
        style::Print(&format!(
            "{0: <20}",
            format!("Stress: {:.0}", employee.resources.stress)
        )),
        cursor::MoveTo(section_start.0, section_start.1 + 5),
    )
    .unwrap();
}

fn draw_current_tasks<W>(w: &mut W, employee_tasks: &Vec<Task>)
where
    W: Write,
{
    let section_start = (5, 10);

    queue!(
        w,
        cursor::MoveTo(section_start.0, section_start.1),
        style::PrintStyledContent("Ongoing tasks".underlined().red()),
    )
    .unwrap();
    let capped_tasks = &employee_tasks[0..5.min(employee_tasks.len())];
    for (i, task) in capped_tasks.iter().enumerate() {
        queue!(
            w,
            cursor::MoveTo(section_start.0, section_start.1 + 2 + i as u16),
            style::PrintStyledContent(format!("[{:?}] ", task.id).green()),
            style::PrintStyledContent(format!("{0: <10}", task.name).white()),
        )
        .unwrap();
        draw_task_progress(w, Color::Green, task.progress(), 10);
    }
}

fn draw_file_name<W>(w: &mut W, file_name: String, okko: &str, anton: &str)
where
    W: Write,
{
    let data_name = if file_name == "okko.txt" { okko } else { anton };

    let data = data_name.split("\n");

    queue!(w, cursor::MoveRight(4), cursor::MoveDown(1)).unwrap();

    for d in data {
        queue!(
            w,
            style::PrintStyledContent(d.to_string().red()),
            cursor::MoveToNextLine(1),
            cursor::MoveRight(5)
        )
        .unwrap();
    }
}

fn draw_time_bar<W>(w: &mut W, start_time: DateTime<Utc>)
where
    W: Write,
{
    let time_seconds = (Utc::now() - start_time).num_seconds() as f32 * TICK_RATE * 10.0;
    let time_in_imaginary_hours = time_seconds / 60.0;
    let days = (time_in_imaginary_hours / 24.0) as i32;
    let hours = (time_in_imaginary_hours - (days as f32 * 24.0)).round() as i32;
    queue!(
        w,
        cursor::MoveToNextLine(1),
        style::PrintStyledContent(
            format!("Days: {}, Hours: {:.2} hours", days, hours)
                .underlined()
                .green()
        ),
        cursor::MoveToNextLine(1),
        cursor::MoveToNextLine(1),
    )
    .unwrap();
}
