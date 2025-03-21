use colored::*;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::Read;
use std::io::{self, BufReader, BufWriter, Write};
use std::path::Path;
use std::{env, process};

pub struct Entry {
    pub todo_entry: String,
    pub done: bool,
}

impl Entry {
    pub fn new(todo_entry: String, done: bool) -> Self {
        Self { todo_entry, done }
    }

    pub fn file_line(&self) -> String {
        let symbol = if self.done { "[*] " } else { "[ ] " };
        format!("{}{}\n", symbol, self.todo_entry,)
    }

    pub fn list_line(&self, number: usize) -> String {
        // Checks if the current task is completed or not...
        let todo_entry = if self.done {
            // DONE
            // If the task is completed, then it prints it with a strikethrough
            self.todo_entry.strikethrough().to_string()
        } else {
            // NOT DONE
            // If the task is not completed yet, then it will print it as it is
            self.todo_entry.clone()
        };
        format!("{number} {todo_entry}\n")
    }

    pub fn read_line(line: &String) -> Self {
        let done = &line[..4] == "[*] ";
        let todo_entry = (&line[4..]).to_string();
        Self { todo_entry, done }
    }

    pub fn raw_line(&self) -> String {
        format!("{}\n", self.todo_entry)
    }
}

pub struct Todo {
    pub todo: Vec<String>,
    pub todo_path: String,
    pub todo_bak: String,
    pub no_backup: bool,
}

impl Todo {
    pub fn new() -> Result<Self, String> {
        let todo_path: String = match env::var("TODO_PATH") {
            Ok(t) => t,
            Err(_) => {
                let home = env::var("HOME").unwrap();

                // Look for a legacy TODO file path
                let legacy_todo = format!("{}/TODO", &home);
                match Path::new(&legacy_todo).exists() {
                    true => legacy_todo,
                    false => format!("{}/.todo", &home),
                }
            }
        };

        let todo_bak: String = match env::var("TODO_BAK_DIR") {
            Ok(t) => t,
            Err(_) => String::from("/tmp/todo.bak"),
        };

        let no_backup = env::var("TODO_NOBACKUP").is_ok();

        let todofile = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&todo_path)
            .expect("Couldn't open the todofile");

        // Creates a new buf reader
        let mut buf_reader = BufReader::new(&todofile);

        // Empty String ready to be filled with TODOs
        let mut contents = String::new();

        // Loads "contents" string with data
        buf_reader.read_to_string(&mut contents).unwrap();

        // Splits contents of the TODO file into a todo vector
        let todo = contents.lines().map(str::to_string).collect();

        // Returns todo
        Ok(Self {
            todo,
            todo_path,
            todo_bak,
            no_backup,
        })
    }

    // Prints every todo saved
    pub fn list(&self) {
        let stdout = io::stdout();
        // Buffered writer for stdout stream
        let mut writer = BufWriter::new(stdout);
        let mut data = String::new();
        // This loop will repeat itself for each task in TODO file
        for (number, task) in self.todo.iter().enumerate() {
            let entry = Entry::read_line(task);

            let number = number + 1;

            let line = entry.list_line(number);
            data.push_str(&line);
        }
        writer
            .write_all(data.as_bytes())
            .expect("Failed to write to stdout");
    }

    // This one is for yall, dmenu chads <3
    pub fn raw(&self, arg: &[String]) {
        if arg.len() > 1 {
            eprintln!("todo raw takes only 1 argument, not {}", arg.len())
        } else if arg.is_empty() {
            eprintln!("todo raw takes 1 argument (done/todo)");
        } else {
            let stdout = io::stdout();
            // Buffered writer for stdout stream
            let mut writer = BufWriter::new(stdout);
            let mut data = String::new();
            let arg = &arg[0];
            // This loop will repeat itself for each task in TODO file
            for task in self.todo.iter() {
                let entry = Entry::read_line(task);
                if entry.done && arg == "done" {
                    data = entry.raw_line();
                } else if !entry.done && arg == "todo" {
                    data = entry.raw_line();
                }

                writer
                    .write_all(data.as_bytes())
                    .expect("Failed to write to stdout");
            }
        }
    }
    // Adds a new todo
    pub fn add(&self, args: &[String]) {
        if args.is_empty() {
            eprintln!("todo add takes at least 1 argument");
            process::exit(1);
        }
        // Opens the TODO file with a permission to:
        let todofile = OpenOptions::new()
            .create(true) // a) create the file if it does not exist
            .append(true) // b) append a line to it
            .open(&self.todo_path)
            .expect("Couldn't open the todofile");

        let mut buffer = BufWriter::new(todofile);
        for arg in args {
            if arg.trim().is_empty() {
                continue;
            }

            // Appends a new task/s to the file
            let entry = Entry::new(arg.to_string(), false);
            let line = entry.file_line();
            buffer
                .write_all(line.as_bytes())
                .expect("unable to write data");
        }
    }

    // Removes a task
    pub fn remove(&self, args: &[String]) {
        if args.is_empty() {
            eprintln!("todo rm takes at least 1 argument");
            process::exit(1);
        }
        // Opens the TODO file with a permission to:
        let todofile = OpenOptions::new()
            .write(true) // a) write
            .truncate(true) // b) truncrate
            .open(&self.todo_path)
            .expect("Couldn't open the todo file");

        let mut buffer = BufWriter::new(todofile);

        for (pos, line) in self.todo.iter().enumerate() {
            if args.contains(&(pos + 1).to_string()) {
                continue;
            }

            let line = format!("{}\n", line);

            buffer
                .write_all(line.as_bytes())
                .expect("unable to write data");
        }
    }

    fn remove_file(&self) {
        match fs::remove_file(&self.todo_path) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error while clearing todo file: {}", e)
            }
        };
    }
    // Clear todo by removing todo file
    pub fn reset(&self) {
        if !self.no_backup {
            match fs::copy(&self.todo_path, &self.todo_bak) {
                Ok(_) => self.remove_file(),
                Err(_) => {
                    eprint!("Couldn't backup the todo file")
                }
            }
        } else {
            self.remove_file();
        }
    }
    pub fn restore(&self) {
        fs::copy(&self.todo_bak, &self.todo_path).expect("unable to restore the backup");
    }

    // Sorts done tasks
    pub fn sort(&self) {
        // Creates a new empty string
        let newtodo: String;

        let mut todo = String::new();
        let mut done = String::new();

        for line in self.todo.iter() {
            let entry = Entry::read_line(line);
            if entry.done {
                let line = format!("{}\n", line);
                done.push_str(&line);
            } else {
                let line = format!("{}\n", line);
                todo.push_str(&line);
            }
        }

        newtodo = format!("{}{}", &todo, &done);
        // Opens the TODO file with a permission to:
        let mut todofile = OpenOptions::new()
            .write(true) // a) write
            .truncate(true) // b) truncrate
            .open(&self.todo_path)
            .expect("Couldn't open the todo file");

        // Writes contents of a newtodo variable into the TODO file
        todofile
            .write_all(newtodo.as_bytes())
            .expect("Error while trying to save the todofile");
    }

    pub fn done(&self, args: &[String]) {
        if args.is_empty() {
            eprintln!("todo done takes at least 1 argument");
            process::exit(1);
        }

        // Opens the TODO file with a permission to overwrite it
        let todofile = OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(&self.todo_path)
            .expect("Couldn't open the todofile");
        let mut buffer = BufWriter::new(todofile);
        let mut data = String::new();

        for (pos, line) in self.todo.iter().enumerate() {
            let mut entry = Entry::read_line(line);
            let line = if args.contains(&(pos + 1).to_string()) {
                entry.done = !entry.done;
                entry.file_line()
            } else {
                format!("{}\n", line)
            };

            data.push_str(&line);
        }
        buffer
            .write_all(data.as_bytes())
            .expect("unable to write data");
    }

    pub fn edit(&self, args: &[String]) {
        if args.is_empty() || args.len() != 2 {
            eprintln!("todo edit takes exact 2 arguments");
            process::exit(1);
        }
        // Opens the TODO file with a permission to overwrite it
        let todofile = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.todo_path)
            .expect("Couldn't open the todofile");
        let mut buffer = BufWriter::new(todofile);

        for (pos, line) in self.todo.iter().enumerate() {
            let line = if args[0].contains(&(pos + 1).to_string()) {
                let mut entry = Entry::read_line(line);
                entry.todo_entry = args[1].clone();
                entry.file_line()
            } else {
                format!("{}\n", line)
            };
            buffer
                .write_all(line.as_bytes())
                .expect("unable to write data");
        }
    }
}

const TODO_HELP: &str = "Usage: todo [COMMAND] [ARGUMENTS]
Todo is a super fast and simple tasks organizer written in rust
Example: todo list
Available commands:
    - add [TASK/s]
        adds new task/s
        Example: todo add \"buy carrots\"
    - edit [INDEX] [EDITED TASK/s]
        edits an existing task/s
        Example: todo edit 1 banana
    - list
        lists all tasks
        Example: todo list
    - done [INDEX]
        marks task as done
        Example: todo done 2 3 (marks second and third tasks as completed)
    - rm [INDEX]
        removes a task
        Example: todo rm 4
    - reset
        deletes all tasks
    - restore 
        restore recent backup after reset
    - sort
        sorts completed and uncompleted tasks
        Example: todo sort
    - raw [todo/done]
        prints nothing but done/incompleted tasks in plain text, useful for scripting
        Example: todo raw done
";
pub fn help() {
    // For readability
    println!("{}", TODO_HELP);
}
