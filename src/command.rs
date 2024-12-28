use crate::example_functions::*;

#[derive(Debug)]
pub struct Command {
    pub command:        String,
    pub parent:         String,
    pub depth:          u32,
    pub description:    String,
    pub action:         Option<fn()>,
}


impl Command {
    pub fn new(command: &str, parent: &str, depth: u32, description: &str, action: Option<fn()>) -> Self {
        Self {
            command:        command.to_string(),
            parent:         parent.to_string(),
            depth,
            description:    description.to_string(),
            action,
        }
    }
}


// root                           
//  ├── send                      [Depth : 0]
//  │    ├── mobile               [Depth : 1]
//  │    │     └── binding        [Depth : 2]
//  │    │           ├── update   [Depth : 3]
//  │    │           └── ack      [Depth : 3]
//  │    │
//  └── show                      [Depth : 0]
//       ├── table                [Depth : 1]
//       └── command              [Depth : 1]

pub fn make_commands() -> Vec<Command> {

    let mut commands = Vec::new();
    // Create the hierarchical structure of commands
    commands.push(Command::new("send",      "root",     0, "Send a command",		None));
    commands.push(Command::new("mobile",    "send",     1, "Mobile commands",		None));
    commands.push(Command::new("binding",   "mobile",   2, "Binding commands",		None));
    commands.push(Command::new("update",    "binding",  3, "Send binding update",	Some(send_mobile_binding_update)));
    commands.push(Command::new("ack",       "binding",  3, "Send binding ack",		Some(send_mobile_binding_ack)));

    commands.push(Command::new("show",      "root",     0, "Show information",		None));
    commands.push(Command::new("table",     "show",     1, "Show the table",		Some(show_table)));
    commands.push(Command::new("command",   "show",     1, "Show commands",		Some(show_command)));

    commands
}