locals {
  env_vars = {
    ENVIRONMENT                  = var.environment
    DB_URL                       = var.db_url
    DB_SCHEMA                    = var.db_schema
    REDIS_URL                    = var.redis_url
    REDIS_SCHEMA                 = var.redis_schema
    RESEND_API_KEY               = var.resend_api_key
    DOMAIN                       = var.domain
    RESEND_AUDIENCE_ID           = var.resend_audience_id
    ADMIN_PASSWORD               = var.admin_password
    TURNSTILE_SITE_KEY           = var.turnstile_site_key
    TURNSTILE_SITE_SECRET        = var.turnstile_site_secret
    CLOUDFLARE_TOKEN_VALUE       = var.cloudflare_token_value
    CLOUDFLARE_ACCESS_KEY_ID     = var.cloudflare_access_key_id
    CLOUDFLARE_ACCESS_KEY_SECRET = var.cloudflare_access_key_secret
    CLOUDFLARE_ENDPOINT          = var.cloudflare_endpoint
    CLOUDFLARE_BUCKET_NAME       = var.cloudflare_bucket_name
    CLOUDINARY_CLOUD_NAME        = var.cloudinary_cloud_name
    CLOUDINARY_API_KEY           = var.cloudinary_api_key
    CLOUDINARY_API_SECRET        = var.cloudinary_api_secret
    GEMINI_API_KEY               = var.gemini_api_key
    GCLOUD_GEOCODING_API_KEY     = var.gcloud_geocoding_api_key
    PORT                         = tostring(var.port)
  }
}

# --- Providers ---

provider "aws" {
  alias  = "us_east"
  region = "us-east-1"
}

provider "aws" {
  alias  = "ap_south"
  region = "ap-southeast-1"
}

provider "aws" {
  alias  = "eu_central"
  region = "eu-central-1"
}

provider "aws" {
  alias  = "ap_south_2"
  region = "ap-southeast-2"
}

# --- IAM role (both regions can share this pattern) ---

data "aws_iam_policy_document" "lambda_assume" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "lambda_global" {
  provider           = aws.us_east
  name               = "portfolio-lambda-role"
  assume_role_policy = data.aws_iam_policy_document.lambda_assume.json
}

resource "aws_iam_role_policy_attachment" "lambda_basic" {
  provider   = aws.us_east
  role       = aws_iam_role.lambda_global.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

# --- Lambda functions ---
# Point to the binary cargo lambda build produced

resource "aws_lambda_function" "portfolio_us" {
  provider      = aws.us_east
  function_name = "portfolio"
  role          = aws_iam_role.lambda_global.arn

  filename         = "../target/lambda/portfolio/bootstrap.zip"
  source_code_hash = filebase64sha256("../target/lambda/portfolio/bootstrap.zip")
  handler          = "bootstrap"
  runtime          = "provided.al2023"
  architectures    = ["arm64"]

  environment {
    variables = local.env_vars
  }
}

resource "aws_lambda_function" "portfolio_ap" {
  provider      = aws.ap_south
  function_name = "portfolio"
  role          = aws_iam_role.lambda_global.arn

  filename         = "../target/lambda/portfolio/bootstrap.zip"
  source_code_hash = filebase64sha256("../target/lambda/portfolio/bootstrap.zip")
  handler          = "bootstrap"
  runtime          = "provided.al2023"
  architectures    = ["arm64"]

  environment {
    variables = local.env_vars
  }
}

resource "aws_lambda_function" "portfolio_eu" {
  provider      = aws.eu_central
  function_name = "portfolio"
  role          = aws_iam_role.lambda_global.arn

  filename         = "../target/lambda/portfolio/bootstrap.zip"
  source_code_hash = filebase64sha256("../target/lambda/portfolio/bootstrap.zip")
  handler          = "bootstrap"
  runtime          = "provided.al2023"
  architectures    = ["arm64"]

  environment {
    variables = local.env_vars
  }
}

resource "aws_lambda_function" "portfolio_ap_2" {
  provider      = aws.ap_south_2
  function_name = "portfolio"
  role          = aws_iam_role.lambda_global.arn

  filename         = "../target/lambda/portfolio/bootstrap.zip"
  source_code_hash = filebase64sha256("../target/lambda/portfolio/bootstrap.zip")
  handler          = "bootstrap"
  runtime          = "provided.al2023"
  architectures    = ["arm64"]

  environment {
    variables = local.env_vars
  }
}

# --- Function URLs (replaces --enable-function-url flag) ---

resource "aws_lambda_function_url" "portfolio_us" {
  provider           = aws.us_east
  function_name      = aws_lambda_function.portfolio_us.function_name
  authorization_type = "NONE"
}

resource "aws_lambda_function_url" "portfolio_ap" {
  provider           = aws.ap_south
  function_name      = aws_lambda_function.portfolio_ap.function_name
  authorization_type = "NONE"
}

resource "aws_lambda_function_url" "portfolio_eu" {
  provider           = aws.eu_central
  function_name      = aws_lambda_function.portfolio_eu.function_name
  authorization_type = "NONE"
}

resource "aws_lambda_function_url" "portfolio_ap_2" {
  provider           = aws.ap_south_2
  function_name      = aws_lambda_function.portfolio_ap_2.function_name
  authorization_type = "NONE"
}


# --- Outputs ---

output "function_url_us" {
  value = aws_lambda_function_url.portfolio_us.function_url
}

output "function_url_ap" {
  value = aws_lambda_function_url.portfolio_ap.function_url
}

output "function_url_eu" {
  value = aws_lambda_function_url.portfolio_eu.function_url
}

output "function_url_ap_2" {
  value = aws_lambda_function_url.portfolio_ap_2.function_url
}
