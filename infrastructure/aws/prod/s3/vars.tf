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

variable "bucket_name" {
  type = string
  default = "puzzle-passport-media"
}


