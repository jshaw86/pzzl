resource "aws_iam_role" "pzzl_lambda_role" {
  name = "pzzl_lambda_execution_role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
      },
    ]
  })
}

resource "aws_iam_policy_attachment" "lambda_logs" {
  name       = "pzzl_lambda_logs"
  roles      = [aws_iam_role.pzzl_lambda_role.name]
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_lambda_function" "pzzl_lambda_function" {
  function_name = "pzzl-server-${var.env_name}"
  timeout       = 5 # seconds
  image_uri     = "${var.repository_url}:${var.version}"
  package_type  = "Image"

  role = aws_iam_role.pzzl_lambda_role.arn

  environment {
    variables = {
      DATABASE_URL = var.database_url
      DATABASE_USER = var.database_user
      DATABASE_PASSWORD = var.database_user
      DATABASE_TIMEOUT = var.database_timeout
    }
  }
}
