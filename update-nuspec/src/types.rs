use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dependency {
    pub name: String,
    pub version: String,
}

impl Dependency {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct DependencyComparisonResult {
    pub updated_references: Vec<Dependency>,
    pub added_references: Vec<Dependency>,
    pub no_changes_references: Vec<Dependency>,
    pub deleted_references: Vec<Dependency>,
    pub outdated_references: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    Completed,
    ProjectNameNotFound,
    ProjectFileNotFound,
}

#[derive(Debug)]
pub struct ProcessResult {
    pub status: ProcessStatus,
    pub project_id: Option<String>,
    pub comparison: Option<DependencyComparisonResult>,
    pub group_comparisons: Vec<GroupComparisonResult>,
}

#[derive(Debug)]
pub struct GroupComparisonResult {
    pub target_framework: String,
    pub comparison: DependencyComparisonResult,
}
