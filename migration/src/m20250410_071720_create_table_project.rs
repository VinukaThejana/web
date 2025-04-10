use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Project {
    Table,
    Id,
    Title,
    Description,
    Tags,
    Url,
    Date,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Project::Table)
                    .if_not_exists()
                    .col(integer(Project::Id).primary_key().auto_increment())
                    .col(string(Project::Title).string_len(255))
                    .col(string(Project::Description).string_len(255))
                    .col(string(Project::Tags).string_len(255))
                    .col(string(Project::Url).string_len(255))
                    .col(string(Project::Date))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Project::Table).to_owned())
            .await?;

        Ok(())
    }
}
