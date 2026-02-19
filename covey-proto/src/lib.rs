//! Messages sent as newline delimited JSON over stdin/out.

/// A request sent by the covey host to a plugin.
pub mod covey_request {
    use covey_schema::id::CommandId;
    use serde::{Deserialize, Serialize};

    use crate::plugin_response;

    #[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    #[serde(transparent)]
    pub struct RequestId(pub u64);

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub struct Request {
        pub id: RequestId,
        pub request: Body,
    }

    impl Request {
        pub fn query(id: RequestId, query: String) -> Self {
            Self {
                id,
                request: Body::Query(Query { text: query }),
            }
        }

        pub fn activate(
            id: RequestId,
            item_id: plugin_response::ListItemId,
            command_id: CommandId,
        ) -> Self {
            Self {
                id,
                request: Body::Activate(Activate {
                    item_id,
                    command_id,
                }),
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub enum Body {
        Query(Query),
        Activate(Activate),
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub struct Query {
        pub text: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub struct Activate {
        pub item_id: plugin_response::ListItemId,
        pub command_id: CommandId,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn serialize() {
            let json = serde_json::to_string(&Request {
                id: RequestId(0),
                request: Body::Query(Query {
                    text: "this is my query".to_owned(),
                }),
            })
            .unwrap();
            assert_eq!(
                json,
                r#"{"id":0,"request":{"query":{"text":"this is my query"}}}"#
            );
        }
    }
}

/// A response sent by the plugin against the a [`covey_request::Request`].
pub mod plugin_response {
    use std::ops::Range;

    use serde::{Deserialize, Serialize};

    use crate::covey_request;

    #[derive(Debug, Serialize, Deserialize, Clone)]
    #[serde(rename_all = "kebab-case")]
    pub struct Response {
        pub request_id: covey_request::RequestId,
        pub response: Body,
    }

    impl Response {
        pub fn write_to_stdout(&self) {
            let json = serde_json::to_string(self).expect("response should be serializable");
            println!("{json}");
        }

        pub fn set_list(request_id: covey_request::RequestId, list: List) -> Self {
            Self {
                request_id,
                response: Body::SetList(list),
            }
        }

        pub fn perform_action(request_id: covey_request::RequestId, action: Action) -> Self {
            Self {
                request_id,
                response: Body::PerformAction(action),
            }
        }

        pub fn finish_activation_response(request_id: covey_request::RequestId) -> Self {
            Self {
                request_id,
                response: Body::FinishActivationResponse,
            }
        }

        pub fn display_error(request_id: covey_request::RequestId, error: String) -> Self {
            Self::perform_action(request_id, Action::DisplayError(error))
        }
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    #[serde(rename_all = "kebab-case")]
    pub enum Body {
        /// Response to [`covey_request::Body::Query`].
        SetList(List),
        /// Response to [`covey_request::Body::Activate`]. Can be sent multiple times.
        PerformAction(Action),
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    #[serde(rename_all = "kebab-case")]
    pub enum Action {
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

    #[derive(Debug, Serialize, Deserialize, Clone, Copy)]
    #[serde(transparent)]
    pub struct ListItemId(pub u64);

    pub use covey_schema::id::CommandId;

    #[derive(Debug, Serialize, Deserialize, Clone)]
    #[serde(rename_all = "kebab-case")]
    pub struct List {
        pub items: Vec<ListItem>,
        pub style: Option<ListStyle>,
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
        pub id: ListItemId,
        pub title: String,
        pub description: String,
        pub available_commands: Vec<CommandId>,
        pub icon: Option<ListItemIcon>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    #[serde(rename_all = "kebab-case")]
    pub enum ListItemIcon {
        Name(String),
        Text(String),
    }
}
