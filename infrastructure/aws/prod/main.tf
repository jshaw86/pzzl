data "aws_route_table" "main_vpc_route_tables" {
  vpc_id = aws_vpc.main.id 
}

resource "aws_vpc" "main" {
  cidr_block = "10.0.0.0/16"
  enable_dns_support = true
  enable_dns_hostnames = true
}

resource "aws_internet_gateway" "main_internet_gateway" {
  vpc_id = aws_vpc.main.id
}


resource "aws_vpc_endpoint" "dynamodb" {
  vpc_id       = aws_vpc.main.id
  service_name = "com.amazonaws.${var.region}.dynamodb"
  vpc_endpoint_type = "Gateway"

  route_table_ids = data.aws_route_table.main_vpc_route_tables[*].id 

}

data "aws_availability_zones" "available" {
  state = "available"
}

resource "aws_subnet" "lambda_subnet_primary" {
  vpc_id     = aws_vpc.main.id
  cidr_block = "10.0.1.0/24"

  availability_zone = data.aws_availability_zones.available.names[0]

   tags = {
       Name = "lambda_subnet_primary"
   }
}

resource "aws_subnet" "lambda_subnet_secondary" {
  vpc_id     = aws_vpc.main.id
  cidr_block = "10.0.2.0/24"

  availability_zone = data.aws_availability_zones.available.names[1]

  tags = {
      Name = "lambda_subnet_secondary"
  }
}

resource "aws_security_group" "lambda_sg" {
  name        = "lambda_sg"
  description = "Security group for Lambda to access RDS"
  vpc_id      = aws_vpc.main.id

  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

 ingress {
    from_port   = 80 
    to_port     = 80 
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
      Name = "lambda_sg"
  }
}

resource "aws_dynamodb_table" "puzzles_users" {
    name           = "puzzles_users"
    read_capacity  = 5
    write_capacity = 5
    hash_key       = "pk"
    range_key       = "sk"

    attribute {
        name = "pk"
        type = "S"
    }

    attribute {
        name = "sk"
        type = "S"
    }

}

resource "aws_lb" "lambda_lb" {
  name               = "lambda-lb"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.lambda_sg.id]
  subnets            = [aws_subnet.lambda_subnet_primary.id, aws_subnet.lambda_subnet_secondary.id]

  enable_deletion_protection = false
}

resource "aws_lb_target_group" "lambda_lb_target_group" {
  name        = "lambda-lb-target-group"
  target_type = "lambda"
  vpc_id      = aws_vpc.main.id 
}

resource "aws_lb_listener" "lambda_lb_80_listener" {
  load_balancer_arn = aws_lb.lambda_lb.arn
  port              = "80"
  protocol          = "HTTP"
  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.lambda_lb_target_group.arn
  }
}

resource "aws_lb_listener_rule" "static" {
  listener_arn = aws_lb_listener.lambda_lb_80_listener.arn
  priority     = 100

  action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.lambda_lb_target_group.arn
  }

  condition {
    path_pattern {
      values = ["/health"]
    }
  }
}
