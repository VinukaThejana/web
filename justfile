set dotenv-load

mod db
mod redis
mod migrate

tailwind:
  bunx @tailwindcss/cli -i ./public/assets/css/global.css -o ./public/assets/css/styles.css --watch
  
