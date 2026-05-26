set dotenv-load

mod db
mod redis
mod migrate

build:
  ulimit -n 8192 && cargo lambda build --release --arm64 --output-format zip --bin portfolio

plan: build
  cd infra && terraform plan

deploy: build
  cd infra && terraform apply -auto-approve

tailwind:
  bunx @tailwindcss/cli -i ./public/assets/css/global.css -o ./public/assets/css/styles.css --watch
