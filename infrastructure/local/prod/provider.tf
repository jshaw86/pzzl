provider "aws" {
  region     = "eu-west-1"

  endpoints {
        dynamodb = "http://localhost:4566"
    }
}
