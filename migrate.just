set dotenv-load

create migration:
  @echo "creating migration"
  sqlx migrate add "{{migration}}"

up:
  @echo "running migrations"
  sqlx migrate run --database-url $(echo $DB_URL)

down:
  @echo "rolling back the last applied migration"
  sqlx migrate revert --database-url $(echo $DB_URL)

reset:
  @echo "resetting database"
  sqlx database drop -y --database-url $(echo $DB_URL)
  sqlx database create --database-url $(echo $DB_URL)
  sqlx migrate run --database-url $(echo $DB_URL)
