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


