use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Post {
    Table,
    Id,
    Title,
    SEOTitle,
    Slug,
    PhotoURL,
    Date,
    Summary,
    Content,
    Tags,
}

const IDX_SLUG: &str = "idx_post_slug";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Post::Table)
                    .if_not_exists()
                    .col(integer(Post::Id).primary_key().auto_increment())
                    .col(string(Post::Title).string_len(255))
                    .col(string(Post::SEOTitle).string_len(255))
                    .col(string(Post::Slug).string_len(255).unique_key())
                    .col(string(Post::PhotoURL).string_len(255))
                    .col(string(Post::Tags).string_len(255))
                    .col(text(Post::Summary))
                    .col(text(Post::Content))
                    .col(integer(Post::Date))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(IDX_SLUG)
                    .table(Post::Table)
                    .col(Post::Slug)
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
                    .name(IDX_SLUG)
                    .table(Post::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Post::Table).to_owned())
            .await?;

        Ok(())
    }
}
