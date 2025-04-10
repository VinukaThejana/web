use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub date: String,
}

impl Project {
    pub fn new(
        id: i32,
        title: String,
        description: String,
        tags: Vec<String>,
        date: String,
    ) -> Self {
        Self {
            id,
            title,
            description,
            tags,
            date,
        }
    }
}

pub trait ToProjects {
    fn to_projects(self) -> Vec<Project>;
}

impl ToProjects for Vec<entity::project::Model> {
    fn to_projects(self) -> Vec<Project> {
        self.into_iter()
            .map(|project| {
                Project::new(
                    project.id,
                    project.title.clone(),
                    project.description.clone(),
                    project.tags.split(",").map(|s| s.to_string()).collect(),
                    project.date,
                )
            })
            .collect()
    }
}
