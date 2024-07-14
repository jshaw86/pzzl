resource "aws_ecr_repository" "pzzl_lambda_repository" {
  name = "pzzl/lambda" // used in lambda to lookup
}
