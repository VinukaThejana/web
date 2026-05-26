variable "environment" {
  type = string
}

variable "db_url" {
  type      = string
  sensitive = true
}

variable "db_schema" {
  type = string
}

variable "redis_url" {
  type      = string
  sensitive = true
}

variable "redis_schema" {
  type = string
}

variable "resend_api_key" {
  type      = string
  sensitive = true
}

variable "domain" {
  type = string
}

variable "resend_audience_id" {
  type = string
}

variable "admin_password" {
  type      = string
  sensitive = true
}

variable "turnstile_site_key" {
  type = string
}

variable "turnstile_site_secret" {
  type      = string
  sensitive = true
}

variable "cloudflare_token_value" {
  type      = string
  sensitive = true
}

variable "cloudflare_access_key_id" {
  type      = string
  sensitive = true
}

variable "cloudflare_access_key_secret" {
  type      = string
  sensitive = true
}

variable "cloudflare_endpoint" {
  type = string
}

variable "cloudflare_bucket_name" {
  type = string
}

variable "cloudinary_cloud_name" {
  type = string
}

variable "cloudinary_api_key" {
  type      = string
  sensitive = true
}

variable "cloudinary_api_secret" {
  type      = string
  sensitive = true
}

variable "gemini_api_key" {
  type      = string
  sensitive = true
}

variable "gcloud_geocoding_api_key" {
  type      = string
  sensitive = true
}

variable "port" {
  type    = number
  default = 8080
}
