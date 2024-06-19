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


  environment {
    variables = {
      
    }
  }
}

