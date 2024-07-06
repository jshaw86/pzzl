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

variable "bucket_name" {
  type = string
  default = "media"
}

variable "env_name" {
  type = string
  default = "prod"
}
variable "repository_url" {
  type = string
}

variable "image_version" {
  type = string
  default = "latest"
}

variable "vercel_ip" {
  type = string
  default = "76.76.21.21"
}

variable "vercel_cname_dns" {
  type = string
  default = "cname.vercel-dns.com"
}
