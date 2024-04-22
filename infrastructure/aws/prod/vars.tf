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

variable "rds_name" {
  type = string
  default = "pzzl"
}

variable "rds_username" {
  type    = string
}

variable "rds_password" {
  type    = string
}
