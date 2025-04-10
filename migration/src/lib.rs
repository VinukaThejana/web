pub use sea_orm_migration::prelude::*;
mod m20250407_124900_create_table_post;
mod m20250410_071720_create_table_project;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250407_124900_create_table_post::Migration),
            Box::new(m20250410_071720_create_table_project::Migration),
        ]
    }
}
