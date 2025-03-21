# todo

A cli todo program written in Rust

## installation

use `cargo install --path .` to compile todo and copy `target/release/todo` to `~/.cargo/bin`

## usage

```Usage: todo [COMMAND] [ARGUMENTS]
todo is a tasks organizer
Example: todo list
Available commands:
    - add [TASK]
        adds new task
        Example: todo add buy carrots
    - edit [INDEX] [EDITED TASK]
        edits an existing task
        Example: todo edit 1 banana
    - list
        lists all tasks
        Example: todo list
    - done [INDEX/s]
        marks tasks as done
        Example: todo done 2 3 (marks second and third tasks as completed)
    - rm [INDEX/s]
        removes a tasks
        Example: todo rm 2 3
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
```
