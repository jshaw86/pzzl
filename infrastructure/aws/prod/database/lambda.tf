resource "aws_iam_role" "pzzl_database_role" {
  name = "pzzl_database_execution_role"

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
  name       = "pzzl_database_logs"
  roles      = [aws_iam_role.pzzl_database_role.name]
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_lambda_function" "pzzl_database_function" {
  function_name = "pzzl-server-${var.env_name}"
  timeout       = 5 # seconds
  image_uri     = "${var.repository_url}:${var.image_version}"
  package_type  = "Image"
  architectures    = ["arm64"]

  role = aws_iam_role.pzzl_database_role.arn

  environment {
    variables = {
      DATABASE_URL = var.database_url
      DATABASE_USER = var.database_user
      DATABASE_PASSWORD = var.database_password
      DATABASE_TIMEOUT = var.database_timeout
      DB_CA_CERT="/etc/ssl/certs/rds-ca-cert.pem" # hardcoded in the dockerfile
    }
  }
}

resource "aws_cloudwatch_event_rule" "once_rule" {
  name                = "one-time-event"
  schedule_expression = var.schedule_time 
}

resource "aws_cloudwatch_event_target" "lambda_target" {
  rule      = aws_cloudwatch_event_rule.once_rule.name
  arn       = aws_lambda_function.pzzl_database_function.arn
  target_id = "MyLambdaFunctionTarget"
}

resource "aws_lambda_permission" "allow_event" {
  statement_id  = "AllowExecutionFromCloudWatch"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.pzzl_database_function.function_name
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.once_rule.arn
}
