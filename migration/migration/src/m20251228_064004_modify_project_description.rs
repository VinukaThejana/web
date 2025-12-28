use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Project {
    Table,
    Description,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Project::Table)
                    .modify_column(ColumnDef::new(Project::Description).text().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Project::Table)
                    .modify_column(
                        ColumnDef::new(Project::Description)
                            .string_len(255)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }
}
