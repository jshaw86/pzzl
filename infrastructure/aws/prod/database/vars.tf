variable "region" {
  type    = string
  default = "us-east-1"
}

variable "env_name" {
    type = string
    default = "prod"
}

variable "access_key" {
  type    = string
}

variable "secret_key" {
  type    = string
}

variable "repository_url" {
  type = string
}

variable "image_version" {
  type = string
  default = "latest"
}

variable "database_url" {
   type = string
}

variable "database_user" {
   type = string
}

variable "database_password" {
   type = string
}
variable "database_timeout" {
   type = number
   default = 30
}

variable "schedule_time" {
    type = string
    default = "cron(0 20 25 5 ? 2024)" # cron(minute hour day month ? year)
}
