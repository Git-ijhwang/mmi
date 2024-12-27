use std::arch::x86_64::_MM_FROUND_CEIL;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use termion::raw::IntoRawMode;
use termion::event::{Event, Key};
use termion::input::TermRead;

use std::io::{stdin, stdout, Read, BufRead};
use std::thread;
use std::time::Duration;

use crossterm::{
    event::{self, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};

#[derive(Debug)]
struct Command {
    command: String,
    parent: String,
    depth: u32,
    description: String,
    action: Option<fn(String)>,
}

impl Command {
    fn new(command: &str, parent: &str, depth: u32, description: &str, action: Option<fn(String)>) -> Self {
        Self {
            command: command.to_string(),
            parent: parent.to_string(),
            depth,
            description: description.to_string(),
            action,
        }
    }
}

#[derive(Debug, Clone)]
struct CommandNode {
    keys: String,
    description: String,
    subcommands: HashMap<String, CommandNode>,
    action: Option<fn(String)>, // 실행할 함수, 필요 시 추가 파라미터 포함
    // children: Vec<CommandNode>,
}

impl CommandNode {
    fn new(keys: &str, description: &str, action: Option<fn(String)>) -> Self {
        Self {
            keys: keys.to_string(),
            description: description.to_string(),
            subcommands: HashMap::new(),
            action,
            // children: Vec::new(),
        }
    }

    fn add_subcommand(&mut self, subcommand: CommandNode) {
        self.subcommands.insert(subcommand.keys.clone(), subcommand);
    }
}


fn make_commands() -> Vec<Command> {

    let mut commands = Vec::new();

    // Create the hierarchical structure of commands
    commands.push(Command::new("send", "root", 0, "Send a command", None));
    commands.push(Command::new("mobile", "send", 1, "Mobile-related commands", None));
    commands.push(Command::new("binding", "mobile", 2, "Binding commands", None));
    commands.push(Command::new("update", "binding", 3, "Send binding update", Some(send_mobile_binding_update)));
    commands.push(Command::new("ack", "binding", 3, "Send binding acknowledgment", Some(send_mobile_binding_ack)));

    commands.push(Command::new("show", "root", 0, "Show information", None));
    commands.push(Command::new("table", "show", 1, "Show the table", Some(show_table)));
    commands.push(Command::new("command", "show", 1, "Show available commands", Some(show_command)));

    commands
}

fn send_mobile_binding_update(args: String) {
    println!("Executing 'send mobile binding update' with args: {:?}", args.to_string());
}

fn send_mobile_binding_ack(args: String) {
    println!("Executing 'send mobile binding ack' with args: {:?}", args.to_string());
}

fn show_table(args: String) {
    println!("Displaying table: {:?}", args.to_string());
}

fn show_command(args: String) {
    println!("Displaying commands: {:?}", args.to_string());
}


fn execute_command(root: &CommandNode, input: &str) {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();

    match find_depth_2nd(&mut root.clone(),parts, 0) {
        Some(found) => {
            if let Some(action) = found.action {
                action("TEST !!!!!".to_string());
            }
        },
        None => {
            println!("\rNo suggestions found for ");
        }
	}
}



fn suggest_next_commands(command_tree: &CommandNode, input: &str) {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let target_key = parts.last().unwrap_or(&"").to_string();

    match find_depth_2nd(&mut command_tree.clone(),parts.clone(), 0) {
        Some(found) => {
            let mut buf = Vec::new();

            for (key, cmd) in &found.subcommands {
                buf.push(key);
            }
            println!(" \n\rSuggestions for '{}': {:?}", target_key, buf);
        },
        None => {
            println!("\rNo suggestions found for '{}'.", parts.last().unwrap());
        }
    }
}


fn find_depth_2nd<'a> (parent: &'a mut CommandNode, command: Vec<&str>, depth: u32)
-> Option<&'a mut CommandNode>
{
	// let temp = parent.clone();
	// find_depth_2nd(parent, command, depth+1);
	let find = parent.subcommands.get_mut(command[depth as usize]);

	match find {
		Some(found) => {
			if depth == command.len() as u32 - 1  {
				return Some(found);
			}
			else if depth < command.len() as u32 - 1  {
				let found = find_depth_2nd(found, command, depth+1);
                return found;
			}
		},
		None => {
			return None;
		}
	}

    None
}


fn insert_in_depth<'a> (parent: &'a mut CommandNode, command: &Command, depth: u32)
-> Option<&'a mut CommandNode>
{
	if &parent.keys == &command.parent || depth == command.depth {
        return Some(parent);
	}

	if depth < command.depth {
		// let (k) =
		for (k, child ) in parent.subcommands.iter_mut() {
			let ret = insert_in_depth( child, command, depth + 1);
			match ret {
				None => {
        			// return Some(parent);
				},
				Some(found) => {
					return Some(found);
				}
			}
		}
	}

    None
}


fn create_and_insert<'a> (parent: &'a mut CommandNode, command: &Command, depth: u32)// -> Option<&'a mut CommandNode>
{
	if depth < command.depth {
		for (k, child ) in parent.subcommands.iter_mut() {
			insert_in_depth( child, command, depth + 1);
		}
	}

	parent.subcommands.insert(
		command.command.clone(),
		CommandNode::new(&command.parent, &command.description, command.action)
	);
}

fn build_command_tree() -> CommandNode {

    let commands = make_commands();
    let mut root = CommandNode::new("root","Root command node", None);

    for command in commands {
        if command.parent == "root" {
            root.subcommands.insert(
                command.command.clone(),
                CommandNode::new(&command.parent, &command.description, command.action),
            );
        } else {
            let parent = insert_in_depth(&mut root, &command, 0);
            match parent {
                None => {
					create_and_insert(&mut root, &command, 0);
                },
                Some(_) => {
                    parent.unwrap().subcommands.insert(
                		command.command.clone(),
                		CommandNode::new(&command.command, &command.description, command.action),
            		);
                }
            }
        }
    }

    root
}

fn main() -> Result<(), String> {
    println!("START Program......");
    // Raw 모드 활성화
    let stdout = io::stdout();
    let mut stdout = stdout.into_raw_mode().map_err(|e| format!("\n\rFailed to enable raw mode: {}", e))?;

    writeln!(stdout, "\n\rType commands. Press 'Tab' to see suggestions. Type 'exit' to quit.").map_err(|e| format!("\n\rFailed to write to stdout: {}", e))?;
    stdout.flush().unwrap();

    let command_tree = Arc::new(Mutex::new(build_command_tree())); // command_tree를 공유할 Arc와 Mutex로 래핑

    let command_tree_clone = Arc::clone(&command_tree);

    // 사용자 입력을 처리하는 쓰레드 생성
    let handle = thread::spawn(move || {
        let stdin = stdin();
        let mut input = String::new();

        print!("\r> "); // 사용자 프롬프트
        stdout.flush().unwrap();
        for evt in stdin.events() {
            let evt = evt.unwrap(); // 이벤트 처리 중 에러는 무시
            match evt {
                Event::Key(Key::Char(c)) if input == "exit" => {
                    writeln!(stdout, "\n\rExiting...").unwrap();
                    break;
                }
                Event::Key(Key::Char('\n')) => {
					if !input.is_empty() {
                    	writeln!(stdout, "\n\rExecuting command: {}", input).unwrap();
                    	execute_command(&command_tree_clone.lock().unwrap(), input.as_str());
                    	input.clear();
                    	write!(stdout, "\r> ").unwrap();
                    	stdout.flush().unwrap();
					}
					else {
						println!("\r>");
                    	stdout.flush().unwrap();
					}
                }
                Event::Key(Key::Backspace) => {
                    input.pop();
                    write!(stdout, "\r> {}", input).unwrap();
                    stdout.flush().unwrap();
                }
                Event::Key(Key::Char('\t')) => {
                    let trimmed_input = input.trim_end();
                    suggest_next_commands(&command_tree_clone.lock().unwrap(), trimmed_input);
                    write!(stdout, "\r> {}", trimmed_input).unwrap();
                    stdout.flush().unwrap();
                }
                Event::Key(Key::Char(c)) => {
                    input.push(c);
                    write!(stdout, "{}", c).unwrap();
                    stdout.flush().unwrap();
                }
                // Event::Key(Key::Esc) |
                Event::Key(Key::Char('\n')) if input == "exit" => {
                    writeln!(stdout, "\n\rExiting command interface.").unwrap();
                    break;
                }
                _ => {}
            }
        }
    });

    handle.join().map_err(|_| "Failed to join input thread.".to_string())?;

    Ok(())
}