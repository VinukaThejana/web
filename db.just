set dotenv-load

connect:
  @echo "connecting to database"
  usql $(echo $DB_URL)
