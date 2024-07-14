data "aws_subnet" "private" {
    filter {
        name   = "tag:Name"
        values = ["private-subnet"]
    }
}

data "aws_security_group" "alb_sg" {
    filter {
        name   = "tag:Name"
        values = ["alb-sg"]
    }
}

data "aws_iam_role" "lambda_role" {
  name = "lambda_role"
}

data "aws_lb_target_group" "lambda_tg" {
  name = "lambda-tg"
}

data "aws_ecr_repository" "lambda_repo" {
  name = "pzzl/lambda"
}

# Create Lambda function
resource "aws_lambda_function" "pzzl_lambda_function" {
  function_name = "pzzl-server-${var.env_name}"
  timeout       = 90 # seconds
  image_uri     = "${data.aws_ecr_repository.lambda_repo.repository_url}:${var.image_version}"
  package_type  = "Image"
  architectures    = ["arm64"]

  vpc_config {
    subnet_ids         = [data.aws_subnet.private.id]
    security_group_ids = [data.aws_security_group.alb_sg.id]
  }

  role = data.aws_iam_role.lambda_role.arn

}

# Grant ALB permission to invoke Lambda
resource "aws_lambda_permission" "alb_invocation" {
  statement_id  = "AllowExecutionFromALB"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.pzzl_lambda_function.function_name
  principal     = "elasticloadbalancing.amazonaws.com"
  source_arn    = data.aws_lb_target_group.lambda_tg.arn
}

# Attach Lambda to target group
resource "aws_lb_target_group_attachment" "lambda_attachment" {
  target_group_arn = data.aws_lb_target_group.lambda_tg.arn
  target_id        = aws_lambda_function.pzzl_lambda_function.arn
}

/*
resource "aws_lambda_provisioned_concurrency_config" "lambda_concurrency" {
  function_name                     = aws_lambda_function.pzzl_lambda_function.function_name
  provisioned_concurrent_executions = 1
  qualifier                         = aws_lambda_function.pzzl_lambda_function.version
}
*/
