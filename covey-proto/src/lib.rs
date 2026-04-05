//! Messages sent as newline delimited JSON over stdin/out.

use std::ops::Range;

pub use covey_schema::id::CommandId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct RequestId(pub u64);

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Request {
    pub id: RequestId,
    pub request: RequestBody,
}

impl Request {
    pub fn query(id: RequestId, query: String) -> Self {
        Self {
            id,
            request: RequestBody::Query(RequestQuery { text: query }),
        }
    }

    pub fn activate(id: RequestId, item_id: ActivationTarget, command_id: CommandId) -> Self {
        Self {
            id,
            request: RequestBody::Activate(RequestActivate {
                target_id: item_id,
                command_id,
            }),
        }
    }

    /// Does not include a newline at the end.
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("request should always be serializable")
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RequestBody {
    Query(RequestQuery),
    Activate(RequestActivate),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RequestQuery {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RequestActivate {
    pub target_id: ActivationTarget,
    pub command_id: CommandId,
}

/// A response sent by the plugin against a [`Request`].
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Response {
    pub request_id: RequestId,
    pub response: ResponseBody,
}

impl Response {
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).expect("response should be serializable")
    }

    pub fn set_list(request_id: RequestId, list: List) -> Self {
        Self {
            request_id,
            response: ResponseBody::SetList(list),
        }
    }

    pub fn perform_action(request_id: RequestId, action: PluginAction) -> Self {
        Self {
            request_id,
            response: ResponseBody::PerformAction(action),
        }
    }

    pub fn display_error(request_id: RequestId, error: String) -> Self {
        Self::perform_action(request_id, PluginAction::DisplayError(error))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum ResponseBody {
    /// Response to [`RequestBody::Query`].
    SetList(List),
    /// Response to [`RequestBody::Activate`]. Can be sent multiple times.
    PerformAction(PluginAction),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum PluginAction {
    Close,
    Copy(String),
    SetInput(Input),
    DisplayError(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Input {
    pub query: String,
    pub selection: Range<usize>,
}

/// A unique ID for a target that can be activated.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct ActivationTarget(pub u64);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct List {
    pub items: Vec<ListItem>,
    pub style: Option<ListStyle>,
    pub id: ActivationTarget,
    /// Commands that are not tied to a particular list item.
    ///
    /// If a list item has an available command with the same command ID, the
    /// list item command will be ran instead of this command.
    pub commands: Vec<CommandId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum ListStyle {
    Rows,
    Grid,
    GridWithColumns(u32),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ListItem {
    pub title: String,
    pub description: String,
    pub icon: Option<ListItemIcon>,
    pub id: ActivationTarget,
    pub commands: Vec<CommandId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum ListItemIcon {
    Name(String),
    Text(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let json = &Request {
            id: RequestId(0),
            request: RequestBody::Query(RequestQuery {
                text: "this is my query".to_owned(),
            }),
        }
        .serialize();
        assert_eq!(
            json,
            r#"{"id":0,"request":{"query":{"text":"this is my query"}}}"#
        );
    }
}
