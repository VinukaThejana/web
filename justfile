set dotenv-load

mod db
mod redis
mod migrate

tailwind:
  bunx @tailwindcss/cli -i ./assets/css/global.css -o ./assets/css/styles.css --watch
  
