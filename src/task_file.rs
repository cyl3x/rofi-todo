use std::fs::File;
use std::io::{self, Read, Seek, Write};
use std::path::PathBuf;

use crate::task::Task;

pub struct TaskFile {
    inner: File,
}

impl TaskFile {
    pub fn new(path: PathBuf) -> io::Result<Self> {
        let inner = File::options()
            .read(true)
            .write(true)
            .truncate(false)
            .create(true)
            .open(path)?;

        Ok(Self { inner })
    }

    pub fn read(&mut self) -> io::Result<Vec<Task>> {
        let mut contents = String::new();

        self.inner.read_to_string(&mut contents)?;

        Ok(contents.lines().map(Task::new).collect())
    }

    pub fn save(&mut self, tasks: &[Task]) -> io::Result<()> {
        let contents = tasks
            .iter()
            .map(|task| task.to_string())
            .collect::<Vec<_>>()
            .join("\n");

        self.inner.seek(io::SeekFrom::Start(0))?;
        self.inner.set_len(0)?;
        self.inner.write_all(contents.as_bytes())?;

        self.inner.flush()
    }
}
