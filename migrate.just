set dotenv-load

create migration:
  @echo "creating migration"
  sea-orm-cli migrate generate "{{migration}}"

up:
  @echo "running migrations"
  sea-orm-cli migrate -u $(echo $DB_URL) --database-schema $(echo $DB_SCHEMA) up

down:
  @echo "rolling back the last applied migration"
  sea-orm-cli migrate -u $(echo $DB_URL) --database-schema $(echo $DB_SCHEMA) down

reset:
  @echo "rolling back all migrations"
  sea-orm-cli migrate -u $(echo $DB_URL) --database-schema $(echo $DB_SCHEMA) reset

generate:
  @echo "generating entities"
  sea-orm-cli generate entity -u $(echo $DB_URL) --database-schema $(echo $DB_SCHEMA) --lib --output-dir entity/src
