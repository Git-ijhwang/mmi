use std::thread;
use std::collections::HashMap;
use std::io::stdin;
use std::io::{self, Write};
use termion::raw::IntoRawMode;
use termion::event::{Event, Key};
use termion::input::TermRead;

use crate::command::*;

#[derive(Debug, Clone)]
struct CommandNode {
    command:        String,
    description:    String,
    subcommands:    HashMap<String, CommandNode>,
    action:         Option<fn()>, // 실행할 함수, 필요 시 추가 파라미터 포함
}

impl CommandNode {
    fn new(command: &str, description: &str, action: Option<fn()>) -> Self {
        Self {
            command:     command.to_string(),
            description: description.to_string(),
            subcommands: HashMap::new(),
            action,
        }
    }

	fn insert_node(&self, command: Command) -> CommandNode {
		let commands = make_commands();
		let mut root = CommandNode::new("root","Root command node", None);

		for command in commands {
			if command.parent == "root" {
				root.subcommands.insert(
					command.command.clone(),
					CommandNode::new(&command.command, &command.description, command.action)
				);
			} else if let Some(parent) = insert_in_depth(&mut root, &command, 0) {
				parent.subcommands.insert(
					command.command.clone(),
					CommandNode::new(&command.command, &command.description, command.action)
				);
			}
		}

    root
	}

}


fn find_in_depth<'a> (node: &'a mut CommandNode, command: Vec<&str>, depth: usize)
-> Option<&'a mut CommandNode>
{
	let pos = depth.min(command.len() - 1);

    // Compare the command at the current depth
    if depth >= command.len() && node.command != command[pos] {
        return None;
    }

	// if the command is found at the current depth
    if depth == command.len() && node.command == command[pos] {
        return Some(node);
    }

    // current depth is less than the target depth
    if depth < command.len() {
        for child in node.subcommands.values_mut() {
            if let Some(found) = find_in_depth(child, command.clone(), depth + 1) {
                return Some(found);
            }
        }
    }

    None
}


fn execute_command(root: &CommandNode, input: &str) {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();

	if let Some(found) = find_in_depth(&mut root.clone(), parts, 0) {
        if let Some(action) = found.action {
            action();
        }
    } else {
        println!("\rNo suggestions found.");
    }
}


fn suggest_next_commands(command_tree: &CommandNode, input: &str) {
	let command = if input.is_empty() { "root" } else { input };
    let parts: Vec<&str> = command.split_whitespace().collect();

    if let Some(found) = find_in_depth(&mut command_tree.clone(), parts, 0) {
        let suggestions: Vec<&String> = found.subcommands.keys().collect();
        println!("\n\rSuggestions: {:?}", suggestions);
    }
	else {
        println!("\rNo suggestions found.");
    }
}


fn insert_in_depth<'a> (node: &'a mut CommandNode, command: &Command, depth: u32)
-> Option<&'a mut CommandNode>
{
	if &node.command == &command.parent && depth  == command.depth     {
        return Some(node);
	}

	if depth < command.depth {
		for child in node.subcommands.values_mut() {
            if let Some(found) = insert_in_depth(child, command, depth + 1) {
                return Some(found);
            }
        }
	}

    None
}

fn build_command_tree() -> CommandNode {
    let commands = make_commands();
    let mut root = CommandNode::new("root","Root command node", None);

    for command in commands {
        if command.parent == "root" {
            root.subcommands.insert(
                command.command.clone(),
                CommandNode::new(&command.command, &command.description, command.action)
            );
        } else if let Some(parent) = insert_in_depth(&mut root, &command, 0) {
            parent.subcommands.insert(
                command.command.clone(),
                CommandNode::new(&command.command, &command.description, command.action)
            );
        }
    }

    root
}


pub fn prompt() -> Result<(), String>
{
    let stdout = io::stdout();
    let mut stdout = stdout.into_raw_mode().map_err(|e| format!("\n\rFailed to enable raw mode: {}", e))?;

    writeln!(stdout, "\n\rType commands. Press 'Tab' to see suggestions. Type 'exit' to quit.").map_err(|e| format!("\n\rFailed to write to stdout: {}", e))?;
    stdout.flush().unwrap();

    let command_tree = build_command_tree();
    let handle = thread::spawn(move || {
        let stdin = stdin();
        let mut input = String::new();

        print!("\r> "); // 사용자 프롬프트
        stdout.flush().unwrap();

        for evt in stdin.events() {
            let evt = evt.unwrap(); // 이벤트 처리 중 에러는 무시
            match evt {

                Event::Key(Key::Char('\n')) => {
					if input.trim().is_empty() {
                        write!(stdout, "\n\r> ").unwrap();
					}
					else if input == "exit" || input == "quit"  {
                        writeln!(stdout, "\n\rExiting command interface.").unwrap();
						break;
					}
					else {
                		writeln!(stdout, "\n\rExecuting command: {}", input).unwrap();
                		execute_command(&command_tree, input.as_str());
                		input.clear();
					}
                	write!(stdout, "\r> ").unwrap();
                }

                Event::Key(Key::Backspace) => {
                    input.pop();
                    write!(stdout, "\r> {}", input).unwrap();
                }

                Event::Key(Key::Char('\t')) => {
                    let trimmed_input = input.trim_end();
                    suggest_next_commands(&command_tree, trimmed_input);
                    write!(stdout, "\r> {}", trimmed_input).unwrap();
                }

                Event::Key(Key::Char(c)) => {
                    input.push(c);
                    write!(stdout, "{}", c).unwrap();
                }

				Event::Key(Key::Ctrl('c')) | Event::Key(Key::Esc) => {
                    input.clear();
                    write!(stdout, "\r> ").unwrap();
                }
                _ => {}
            }
            stdout.flush().unwrap();
        }
    });

    handle.join().map_err(|_| "Failed to join input thread.".to_string())?;

    Ok(())
}