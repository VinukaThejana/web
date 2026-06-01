use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Experience {
    pub id: i32,
    pub title: String,
    pub company: String,
    pub description: String,
    pub tags: Vec<String>,
    pub date: String,
}

impl Experience {
    pub fn new(
        id: i32,
        title: String,
        company: String,
        description: String,
        tags: Vec<String>,
        date: String,
    ) -> Self {
        Self {
            id,
            title,
            company,
            description,
            tags,
            date,
        }
    }
}

#[derive(sqlx::FromRow, Clone, Debug, Serialize, Deserialize)]
pub struct ExperienceModel {
    pub id: i32,
    pub title: String,
    pub company: String,
    pub description: String,
    pub tags: String,
    pub date: String,
}

pub trait ToExperiences {
    fn to_experiences(self) -> Vec<Experience>;
}

impl ToExperiences for Vec<ExperienceModel> {
    fn to_experiences(self) -> Vec<Experience> {
        self.into_iter()
            .map(|exp| {
                Experience::new(
                    exp.id,
                    exp.title.clone(),
                    exp.company.clone(),
                    exp.description.clone(),
                    exp.tags.split(",").map(|s| s.trim().to_string()).collect(),
                    exp.date,
                )
            })
            .collect()
    }
}
