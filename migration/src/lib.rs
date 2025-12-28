pub use sea_orm_migration::prelude::*;
mod m20250407_124900_create_table_post;
mod m20250410_071720_create_table_project;
mod m20250425_182004_create_table_short;
mod m20251228_064016_modify_project_description;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250407_124900_create_table_post::Migration),
            Box::new(m20250410_071720_create_table_project::Migration),
            Box::new(m20250425_182004_create_table_short::Migration),
            Box::new(m20251228_064016_modify_project_description::Migration),
        ]
    }
}
