variable "region" {
  type    = string
  default = "us-east-1"
}

variable "access_key" {
  type    = string
}

variable "secret_key" {
  type    = string
}

variable "env_name" {
  type = string
  default = "prod"
}

variable "image_version" {
  type = string
  default = "1.3.0"
}

