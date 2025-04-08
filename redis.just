set dotenv-load

connect:
  @echo "connecting to redis"
  iredis --url $(echo $REDIS_URL)
