use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub url: String,
    pub tags: Vec<String>,
    pub date: String,
}

impl Project {
    pub fn new(
        id: i32,
        title: String,
        description: String,
        url: String,
        tags: Vec<String>,
        date: String,
    ) -> Self {
        Self {
            id,
            title,
            description,
            url,
            tags,
            date,
        }
    }
}

#[derive(sqlx::FromRow, Clone, Debug, Serialize, Deserialize)]
pub struct ProjectModel {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub tags: String,
    pub url: String,
    pub date: String,
}

pub trait ToProjects {
    fn to_projects(self) -> Vec<Project>;
}

impl ToProjects for Vec<ProjectModel> {
    fn to_projects(self) -> Vec<Project> {
        self.into_iter()
            .map(|project| {
                Project::new(
                    project.id,
                    project.title.clone(),
                    project.description.clone(),
                    project.url.clone(),
                    project
                        .tags
                        .split(",")
                        .map(|s| s.chars().filter(|c| !c.is_whitespace()).collect())
                        .collect(),
                    project.date,
                )
            })
            .collect()
    }
}
