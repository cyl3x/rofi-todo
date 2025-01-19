use std::fmt::Write;
use std::path::PathBuf;

use rofi_mode::{Action, Event};
use strum::{EnumIter, FromRepr, IntoEnumIterator};
use todo_txt::{task, Priority};

#[derive(Clone, Copy, PartialEq)]
enum Menu {
    Tasks,
    ModifyTask(usize, Option<ModifyOption>),
}

#[repr(usize)]
#[derive(FromRepr, EnumIter, Clone, Copy, PartialEq)]
enum ModifyOption {
    Done = 0,
    Subject = 1,
    Priority = 2,
    Delete = 3,
    Back = 4,
}

struct Mode<'rofi> {
    api: rofi_mode::Api<'rofi>,
    path: PathBuf,
    tasks: Vec<task::Simple>,
    error: Option<String>,
    menu: Menu,
}

impl<'rofi> Mode<'rofi> {
    pub fn new(api: rofi_mode::Api<'rofi>, path: PathBuf) -> Self {
        Self {
            api,
            path,
            tasks: vec![],
            error: None,
            menu: Menu::Tasks,
        }
    }

    pub fn read(&mut self) -> std::io::Result<()> {
        if std::fs::exists(&self.path)? {
            let content = std::fs::read_to_string(&self.path)?;

            self.tasks = content.lines().map(todo_txt::parser::task).collect();
        }

        Ok(())
    }

    pub fn save(&self) -> std::io::Result<()> {
        let contents = self.tasks.iter().fold(String::new(), |mut s, task| {
            writeln!(&mut s, "{task}").unwrap();
            s
        });

        std::fs::write(&self.path, contents)
    }

    pub fn formatted_task(&self, line: usize) -> String {
        let task = &self.tasks[line];

        let project = task
            .projects
            .iter()
            .map(|p| format!("<span fgcolor='green'>+{p}</span>"))
            .collect::<Vec<String>>()
            .join(" ");

        let context = task
            .contexts
            .iter()
            .map(|p| format!("<span fgcolor='orange'>@{p}</span>"))
            .collect::<Vec<String>>()
            .join(" ");

        let mut subject = clean_subject(task);
        if task.finished {
            subject = format!("<span fgcolor='gray'>{subject}</span>");
        }

        let priority = task.priority.to_string();

        format!(
            "{} {} {} {}",
            format_args!("<span fgcolor='red'><b>{priority}</b></span>"),
            subject,
            context,
            project,
        )
    }

    pub fn switch_menu(&mut self, menu: Menu) -> Action {
        match menu {
            Menu::Tasks => self.api.set_display_name("tasks"),
            Menu::ModifyTask(_, None) => self.api.set_display_name("modify"),
            Menu::ModifyTask(_, Some(_)) => self.api.set_display_name("edit"),
        }

        self.menu = menu;

        Action::Reset
    }

    pub fn menu(&self, line: usize) -> String {
        match self.menu {
            Menu::Tasks => self.formatted_task(line),
            Menu::ModifyTask(task, modify) => match modify {
                None => match ModifyOption::from_repr(line).unwrap() {
                    ModifyOption::Done => match self.tasks[task].finished {
                        true => format!("{}. Mark as undone", line + 1),
                        false => format!("{}. Mark as done", line + 1),
                    },
                    ModifyOption::Subject => format!("{}. Edit task", line + 1),
                    ModifyOption::Priority => format!("{}. Edit priority", line + 1),
                    ModifyOption::Delete => format!("{}. Delete", line + 1),
                    ModifyOption::Back => "&#60;- Back".into(),
                },
                Some(option) => match option {
                    ModifyOption::Priority => match line {
                        0 => "Reset priority".into(),
                        _ => Priority::from((line - 1) as u8).to_string(),
                    },
                    ModifyOption::Delete => match line {
                        0 => "Confirm".into(),
                        1 => "Cancel".into(),
                        _ => String::new(),
                    },
                    _ => String::new(),
                },
            },
        }
    }

    pub fn handle_ok(&mut self, line: usize, input: &mut rofi_mode::String) -> Action {
        match self.menu {
            Menu::Tasks => self.switch_menu(Menu::ModifyTask(line, None)),
            Menu::ModifyTask(task, modify) => match modify {
                None => match ModifyOption::from_repr(line).unwrap() {
                    ModifyOption::Done => {
                        match self.tasks[task].finished {
                            true => self.tasks[task].uncomplete(),
                            false => self.tasks[task].complete(),
                        }

                        Action::Reload
                    }
                    ModifyOption::Subject => {
                        let _ = std::mem::replace(input, self.tasks[task].subject.clone().into());

                        self.switch_menu(Menu::ModifyTask(task, Some(ModifyOption::Subject)));

                        Action::Reload
                    }
                    ModifyOption::Priority => {
                        self.switch_menu(Menu::ModifyTask(task, Some(ModifyOption::Priority)))
                    }
                    ModifyOption::Delete => {
                        self.switch_menu(Menu::ModifyTask(task, Some(ModifyOption::Delete)))
                    }
                    ModifyOption::Back => self.switch_menu(Menu::Tasks),
                },
                Some(option) => {
                    match option {
                        ModifyOption::Subject => {
                            let edited = todo_txt::parser::task(input);

                            self.tasks[task].subject = edited.subject;
                            self.tasks[task].projects = edited.projects;
                            self.tasks[task].hashtags = edited.hashtags;

                            std::mem::take(input);
                        }
                        ModifyOption::Priority => {
                            self.tasks[task].priority = match line {
                                0 => Priority::lowest(),
                                _ => Priority::from((line - 1) as u8),
                            }
                        }
                        ModifyOption::Delete => if line == 0 {
                            self.tasks.remove(task);
                            return self.switch_menu(Menu::Tasks);
                        },
                        _ => (),
                    }

                    self.switch_menu(Menu::ModifyTask(task, None))
                }
            },
        }
    }

    pub fn handle_delete(&mut self, line: usize) -> Action {
        if self.menu == Menu::Tasks {
            self.tasks.remove(line);
        }

        Action::Reload
    }

    pub fn handle_alt_ok(&mut self, line: usize, input: &mut rofi_mode::String) -> Action {
        match self.menu {
            Menu::Tasks => {
                self.tasks[line].complete();

                Action::Reload
            }
            _ => self.handle_ok(line, input),
        }
    }

    pub fn handle_cancel(&mut self) -> Action {
        match self.menu {
            Menu::Tasks => Action::Exit,
            Menu::ModifyTask(task, modify) => match modify {
                Some(_) => self.switch_menu(Menu::ModifyTask(task, None)),
                None => self.switch_menu(Menu::Tasks),
            },
        }
    }
}

impl<'rofi> rofi_mode::Mode<'rofi> for Mode<'rofi> {
    const NAME: &'static str = "todo\0";

    fn init(api: rofi_mode::Api<'rofi>) -> Result<Self, ()> {
        env_logger::builder().init();

        let config = todo_txt::Config::from_env();
        let path = PathBuf::from(config.todo_file);

        let mut data = Self::new(api, path);

        if let Err(err) = data.read() {
            log::error!("{err:?}");
            data.error = Some(format!("{err:?}"));
        };

        data.switch_menu(Menu::Tasks);

        Ok(data)
    }

    fn entries(&mut self) -> usize {
        match self.menu {
            Menu::Tasks => self.tasks.len(),
            Menu::ModifyTask(_, modify) => match modify {
                None => ModifyOption::iter().len(),
                Some(option) => match option {
                    ModifyOption::Priority => 25,
                    ModifyOption::Delete => 2,
                    _ => 0,
                },
            },
        }
    }

    fn entry_content(&self, line: usize) -> rofi_mode::String {
        rofi_mode::format!("{}", self.menu(line))
    }

    fn react(&mut self, event: Event, input: &mut rofi_mode::String) -> rofi_mode::Action {
        let action = match event {
            Event::DeleteEntry { selected } => self.handle_delete(selected),
            Event::Ok { alt, selected } => match alt {
                false => self.handle_ok(selected, input),
                true => self.handle_alt_ok(selected, input),
            },
            Event::CustomInput { .. } => self.handle_ok(0, input),
            Event::Cancel { .. } => self.handle_cancel(),
            _ => Action::Exit,
        };

        if action == Action::Exit {
            if let Err(err) = self.save() {
                log::error!("{err:?}");
                self.error = Some(format!("{err:?}"));
                return Action::Reload;
            };
        }

        action
    }

    fn matches(&self, line: usize, matcher: rofi_mode::Matcher<'_>) -> bool {
        match self.menu {
            Menu::Tasks => matcher.matches(&self.tasks[line].subject),
            _ => matcher.matches(&self.menu(line)),
        }
    }

    fn entry_style(&self, _line: usize) -> rofi_mode::Style {
        rofi_mode::Style::MARKUP
    }

    fn message(&mut self) -> rofi_mode::String {
        if let Some(error) = &self.error {
            return rofi_mode::format!("<span fgcolor='red'>{}</span>", error);
        }

        match self.menu {
            Menu::Tasks => rofi_mode::String::new(),
            Menu::ModifyTask(task, modify) => match modify {
                None => rofi_mode::format!("Task: {}", self.formatted_task(task)),
                Some(option) => match option {
                    ModifyOption::Delete => {
                        rofi_mode::format!("Delete: {}", self.formatted_task(task))
                    }
                    ModifyOption::Priority => {
                        rofi_mode::format!("Priority: {}", self.formatted_task(task))
                    }
                    _ => rofi_mode::format!("Edit: {}", self.formatted_task(task)),
                },
            },
        }
    }
}

fn clean_subject(task: &task::Simple) -> String {
    let mut subject = task.subject.clone();

    for project in &task.projects {
        subject = subject.replace(&format!(" +{project}"), "");
    }

    for context in &task.contexts {
        subject = subject.replace(&format!(" @{context}"), "");
    }

    for hashtag in &task.hashtags {
        subject = subject.replace(&format!(" #{hashtag}"), "");
    }

    subject
}

rofi_mode::export_mode!(Mode);
