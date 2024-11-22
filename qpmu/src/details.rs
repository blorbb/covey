use crate::plugin::{proto, Plugin};

/// Details of a plugin.
#[derive(Debug)]
pub struct Details {
    plugin: Plugin,
    proto: proto::DetailsResponse,
}

impl Details {
    pub fn name(&self) -> Option<&str> {
        self.proto.name.as_deref()
    }

    pub fn author(&self) -> &[String] {
        &self.proto.authors
    }

    pub fn repository(&self) -> Option<&str> {
        self.proto.repository.as_deref()
    }

    pub fn description(&self) -> Option<&str> {
        self.proto.description.as_deref()
    }

    pub fn plugin(&self) -> Plugin {
        self.plugin
    }

    pub(crate) fn from_proto(plugin: Plugin, proto: proto::DetailsResponse) -> Self {
        Self { plugin, proto }
    }
}
