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

variable "database_user" {
   type = string
}

variable "database_name" {
   type = string
}

variable "database_password" {
   type = string
}
variable "database_timeout" {
   type = number
   default = 30
}


