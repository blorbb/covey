use super::Plugin;
use crate::{plugin::bindings, Input};

#[derive(Debug)]
pub enum Action {
    Close,
    RunCommand(String, Vec<String>),
    RunShell(String),
    Copy(String),
    SetInputLine(Input),
}

impl Action {
    pub fn from_wit_action(plugin: Plugin, action: bindings::Action) -> Self {
        match action {
            bindings::Action::Close => Self::Close,
            bindings::Action::RunCommand((cmd, args)) => Self::RunCommand(cmd, args),
            bindings::Action::RunShell(str) => Self::RunShell(str),
            bindings::Action::Copy(str) => Self::Copy(str),
            bindings::Action::SetInputLine(input_line) => {
                let mut input = Input {
                    contents: input_line.query,
                    selection: (input_line.range.lower_bound, input_line.range.upper_bound),
                };
                input.prefix_with(&plugin.prefix());
                Self::SetInputLine(input)
            }
        }
    }
}
