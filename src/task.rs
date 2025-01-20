use std::ops::{Deref, DerefMut};

use todo_txt::task::Simple;

pub struct Task(Simple);

impl Deref for Task {
    type Target = Simple;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Task {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Task {
    pub fn new(line: &str) -> Self {
        let inner = todo_txt::parser::task(line);

        Self(inner)
    }

    pub fn update(&mut self, line: &str) {
        let edited = todo_txt::parser::task(line);

        self.subject = edited.subject;
        self.projects = edited.projects;
        self.hashtags = edited.hashtags;
    }

    pub fn pango_string(&self) -> String {
        format!(
            "{} {} {} {}",
            self.pango_string_priority(),
            self.pango_string_subject(),
            self.pango_string_contexts(),
            self.pango_string_projects(),
        )
    }

    pub fn pango_string_subject(&self) -> String {
        let mut subject = self.stripped_subject();
        if self.finished {
            subject = format!("<span fgcolor='gray'>{subject}</span>");
        }

        subject
    }

    pub fn pango_string_priority(&self) -> String {
        format!("<span fgcolor='red'><b>{}</b></span>", self.priority)
    }

    pub fn pango_string_projects(&self) -> String {
        self.projects
            .iter()
            .map(|p| format!("<span fgcolor='green'>+{p}</span>"))
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn pango_string_contexts(&self) -> String {
        self.contexts
            .iter()
            .map(|p| format!("<span fgcolor='orange'>@{p}</span>"))
            .collect::<Vec<String>>()
            .join(" ")
    }

    pub fn stripped_subject(&self) -> String {
        let mut subject = self.subject.clone();

        for project in &self.projects {
            subject = subject.replace(&format!(" +{project}"), "");
        }

        for context in &self.contexts {
            subject = subject.replace(&format!(" @{context}"), "");
        }

        for hashtag in &self.hashtags {
            subject = subject.replace(&format!(" #{hashtag}"), "");
        }

        subject
    }
}
