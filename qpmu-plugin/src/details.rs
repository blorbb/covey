use crate::proto;

/// Details of this plugin.
pub struct Details {
    name: Option<String>,
    authors: Vec<String>,
    repository: Option<String>,
    description: Option<String>,
}

impl Details {
    /// Creates a new empty details struct.
    ///
    /// You should call the builder methods to add information.
    pub fn new() -> Self {
        Self { name: None, authors: vec![], repository: None, description: None }
    }

    /// Sets the plugin's name.
    ///
    /// Should be a single line.
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Adds an author to the list of authors.
    ///
    /// This can be called multiple times to add multiple authors.
    pub fn author(mut self, author: String) -> Self {
        self.authors.push(author);
        self
    }

    /// Sets the repository URL.
    pub fn repository(mut self, repo: String) -> Self {
        self.repository = Some(repo);
        self
    }

    /// Sets the description.
    ///
    /// Can be multiple lines long.
    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub(crate) fn into_proto(self) -> proto::DetailsResponse {
        proto::DetailsResponse {
            name: self.name,
            authors: self.authors,
            repository: self.repository,
            description: self.description,
        }
    }
}
