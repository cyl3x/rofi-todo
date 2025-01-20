use config::Config;
use rofi_mode::{Action, Event};
use strum::{EnumIter, FromRepr, IntoEnumIterator};
use task::Task;
use task_file::TaskFile;
use todo_txt::Priority;

mod config;
mod task;
mod task_file;

#[derive(Clone, PartialEq)]
enum Menu {
    Tasks,
    ModifyTask(usize, Option<ModifyOption>),
    AddTask(Task),
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
    file: Option<TaskFile>,
    config: Config,
    tasks: Vec<Task>,
    error: Option<String>,
    menu: Menu,
}

impl<'rofi> Mode<'rofi> {
    pub fn new(api: rofi_mode::Api<'rofi>) -> Self {
        let mut state = Self {
            api,
            file: None,
            config: Config::new(),
            tasks: vec![],
            error: None,
            menu: Menu::Tasks,
        };

        log::info!(
            "todo.txt file path: {}",
            state.config.file.to_string_lossy()
        );

        let file = TaskFile::new(&state.config.file)
            .and_then(|mut file| file.read().map(|tasks| (file, tasks)));

        match file {
            Ok((file, tasks)) => {
                state.file = Some(file);
                state.tasks = tasks;
            }
            Err(err) => {
                log::error!("{err:?}");
                state.error = Some(format!("{err:?}"));
            }
        }

        state.switch_menu(Menu::Tasks);

        state
    }

    pub fn save(&mut self) {
        let Some(file) = &mut self.file else {
            return;
        };

        if let Err(err) = file.save(&self.tasks) {
            log::error!("{err:?}");
            self.error = Some(format!("{err:?}"));
        }
    }

    pub fn switch_menu(&mut self, menu: Menu) -> Action {
        match menu {
            Menu::Tasks => self.api.set_display_name("tasks"),
            Menu::ModifyTask(_, None) => self.api.set_display_name("modify"),
            Menu::ModifyTask(_, Some(_)) => self.api.set_display_name("edit"),
            Menu::AddTask(_) => self.api.set_display_name("add"),
        }

        self.menu = menu;

        Action::Reset
    }

    pub fn menu(&self, line: usize) -> String {
        match self.menu {
            Menu::Tasks => self.tasks[line].pango_string(&self.config),
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
            Menu::AddTask(_) => match line {
                0 => "Confirm".into(),
                1 => "Cancel".into(),
                _ => String::new(),
            },
        }
    }

    pub fn handle_ok(&mut self, line: usize, input: &mut rofi_mode::String) -> Action {
        match self.menu.clone() {
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
                        ModifyOption::Priority => {
                            self.tasks[task].priority = match line {
                                0 => Priority::lowest(),
                                _ => Priority::from((line - 1) as u8),
                            }
                        }
                        ModifyOption::Delete => {
                            if line == 0 {
                                self.tasks.remove(task);
                                return self.switch_menu(Menu::Tasks);
                            }
                        }
                        _ => (),
                    }

                    self.switch_menu(Menu::ModifyTask(task, None))
                }
            },
            Menu::AddTask(task) => {
                if line == 0 {
                    self.tasks.push(task);
                }

                self.switch_menu(Menu::Tasks)
            }
        }
    }

    pub fn handle_custom_ok(&mut self, input: &mut rofi_mode::String) -> Action {
        match self.menu {
            Menu::Tasks => {
                let task = Task::new(&std::mem::take(input));
                self.switch_menu(Menu::AddTask(task))
            }
            Menu::ModifyTask(task, Some(ModifyOption::Subject)) => {
                self.tasks[task].update(input);
                std::mem::take(input);

                self.switch_menu(Menu::ModifyTask(task, None))
            }
            _ => Action::Reload,
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
                match self.tasks[line].finished {
                    true => self.tasks[line].uncomplete(),
                    false => self.tasks[line].complete(),
                }

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
            Menu::AddTask(_) => self.switch_menu(Menu::Tasks),
        }
    }
}

impl<'rofi> rofi_mode::Mode<'rofi> for Mode<'rofi> {
    const NAME: &'static str = "todo\0";

    fn init(api: rofi_mode::Api<'rofi>) -> Result<Self, ()> {
        env_logger::builder().init();

        Ok(Self::new(api))
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
            Menu::AddTask(_) => 2,
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
            Event::CustomInput { .. } => self.handle_custom_ok(input),
            Event::Cancel { .. } => self.handle_cancel(),
            _ => Action::Exit,
        };

        if action == Action::Exit {
            self.save();
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

        match &self.menu {
            Menu::Tasks => rofi_mode::String::new(),
            Menu::ModifyTask(task, modify) => match modify {
                None => {
                    rofi_mode::format!("Task: {}", self.tasks[*task].pango_string(&self.config))
                }
                Some(option) => match option {
                    ModifyOption::Delete => {
                        rofi_mode::format!(
                            "Delete: {}",
                            self.tasks[*task].pango_string(&self.config)
                        )
                    }
                    ModifyOption::Priority => {
                        rofi_mode::format!(
                            "Priority: {}",
                            self.tasks[*task].pango_string(&self.config)
                        )
                    }
                    _ => {
                        rofi_mode::format!("Edit: {}", self.tasks[*task].pango_string(&self.config))
                    }
                },
            },
            Menu::AddTask(task) => rofi_mode::format!("Add: {}", task.pango_string(&self.config)),
        }
    }
}

rofi_mode::export_mode!(Mode);
