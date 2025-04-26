use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Short {
    Table,
    Id,
    LongUrl,
    Key,
    Views,
    Description,
    CreatedAt,
}

const IDX_KEY: &str = "idx_short_key";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Short::Table)
                    .if_not_exists()
                    .col(pk_auto(Short::Id))
                    .col(string(Short::LongUrl))
                    .col(string(Short::Key).unique_key())
                    .col(string(Short::Description))
                    .col(integer(Short::Views).extra("DEFAULT 0"))
                    .col(date_time(Short::CreatedAt).extra("DEFAULT CURRENT_TIMESTAMP"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(IDX_KEY)
                    .table(Short::Table)
                    .col(Short::Key)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name(IDX_KEY)
                    .table(Short::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Short::Table).to_owned())
            .await?;

        Ok(())
    }
}
