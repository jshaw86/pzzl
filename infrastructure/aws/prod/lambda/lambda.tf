data "aws_caller_identity" "current" {}

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
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaVPCAccessExecutionRole"
}

data "aws_subnet" "lambda_subnet_primary" {
  filter {
    name   = "tag:Name"
    values = ["lambda_subnet_primary"]
  }
}

data "aws_subnet" "lambda_subnet_secondary" {
  filter {
    name   = "tag:Name"
    values = ["lambda_subnet_secondary"]
  }
}


data "aws_security_group" "lambda_sg" {
  filter {
    name   = "tag:Name"
    values = ["lambda_sg"]
  }

}

resource "aws_iam_policy" "lambda_dynamodb_policy" {
  name = "lambda_dynamodb_policy"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "dynamodb:GetItem",
          "dynamodb:Query",
          "dynamodb:Scan",
          "dynamodb:PutItem",
          "dynamodb:UpdateItem",
          "dynamodb:DeleteItem",
          "dynamodb:ListTables"
        ]
        Resource = "arn:aws:dynamodb:${var.region}:${data.aws_caller_identity.current.account_id}:table/*"
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "lambda_dynamodb_attach" {
  role       = aws_iam_role.pzzl_lambda_role.name
  policy_arn = aws_iam_policy.lambda_dynamodb_policy.arn
}

resource "aws_lambda_function" "pzzl_lambda_function" {
  function_name = "pzzl-server-${var.env_name}"
  timeout       = 90 # seconds
  image_uri     = "${var.repository_url}:${var.image_version}"
  package_type  = "Image"
  architectures    = ["arm64"]

  vpc_config {
    subnet_ids         = [data.aws_subnet.lambda_subnet_primary.id, data.aws_subnet.lambda_subnet_secondary.id]
    security_group_ids = [data.aws_security_group.lambda_sg.id]
  }

  role = aws_iam_role.pzzl_lambda_role.arn

}

data "aws_lb_target_group" "lambda-lb-target-group" {
    name="lambda-lb-target-group"
}

resource "aws_lambda_permission" "lambda_lb_permission" {
  statement_id  = "AllowExecutionFromALB"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.pzzl_lambda_function.function_name
  principal     = "elasticloadbalancing.amazonaws.com"
  source_arn    = data.aws_lb_target_group.lambda-lb-target-group.arn
}



resource "aws_lb_target_group_attachment" "lambda_lb_target_group_attachment" {
  target_group_arn = data.aws_lb_target_group.lambda-lb-target-group.arn
  target_id        = aws_lambda_function.pzzl_lambda_function.arn
  depends_on       = [aws_lambda_permission.lambda_lb_permission]
}

