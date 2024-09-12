# Define the VPC
resource "aws_vpc" "main" {
  cidr_block = "10.0.0.0/16"
  assign_generated_ipv6_cidr_block = true
  tags = {
    Name = "main-vpc"
  }
}

# Define public subnet
resource "aws_subnet" "publica" {
  vpc_id     = aws_vpc.main.id
  ipv6_cidr_block      = cidrsubnet(aws_vpc.main.ipv6_cidr_block, 8, 0)
  cidr_block           = cidrsubnet(aws_vpc.main.cidr_block, 4, 10)
  map_public_ip_on_launch = true
  availability_zone = "us-east-1a"
  tags = {
    Name = "publica-subnet"
  }
}

resource "aws_subnet" "publicb" {
  vpc_id     = aws_vpc.main.id
  ipv6_cidr_block      = cidrsubnet(aws_vpc.main.ipv6_cidr_block, 8, 1)
  cidr_block           = cidrsubnet(aws_vpc.main.cidr_block, 4, 11)
  map_public_ip_on_launch = true
  availability_zone = "us-east-1b"
  tags = {
    Name = "publicb-subnet"
  }
}

# Define private subnet
resource "aws_subnet" "private" {
  vpc_id     = aws_vpc.main.id
  ipv6_cidr_block      = cidrsubnet(aws_vpc.main.ipv6_cidr_block, 8, 3)
  cidr_block           = cidrsubnet(aws_vpc.main.cidr_block, 4, 12)
  availability_zone = "us-east-1a"
  tags = {
    Name = "private-subnet" // used in the lambda to lookup
  }
}

# Create an Internet Gateway
resource "aws_internet_gateway" "gw" {
  vpc_id = aws_vpc.main.id
  tags = {
    Name = "main-gw"
  }
}

# Create route tables
resource "aws_route_table" "publica" {
  vpc_id = aws_vpc.main.id
  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.gw.id
  }
  route {
    ipv6_cidr_block = "::/0"
    gateway_id = aws_internet_gateway.gw.id
  }
  tags = {
    Name = "publica-rt"
  }
}

# Create route tables
resource "aws_route_table" "publicb" {
  vpc_id = aws_vpc.main.id
  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.gw.id
  }
  route {
    ipv6_cidr_block = "::/0"
    gateway_id = aws_internet_gateway.gw.id
  }
  tags = {
    Name = "publica-rt"
  }
}

# Associate route tables with subnets
resource "aws_route_table_association" "publica" {
  subnet_id      = aws_subnet.publica.id
  route_table_id = aws_route_table.publica.id
}

# Associate route tables with subnets
resource "aws_route_table_association" "publicb" {
  subnet_id      = aws_subnet.publicb.id
  route_table_id = aws_route_table.publicb.id
}

#ALB
# Create security group for ALB
resource "aws_security_group" "alb_sg" {
  vpc_id = aws_vpc.main.id
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  ingress {
    from_port = 80 
    to_port = 80
    protocol = "tcp"
    ipv6_cidr_blocks = ["::/0"]
  }

  ingress {
    from_port   = 443 
    to_port     = 443 
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  ingress {
    from_port = 443 
    to_port = 443 
    protocol = "tcp"
    ipv6_cidr_blocks = ["::/0"]
  }
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port = 0
    to_port = 0
    protocol = "-1"
    ipv6_cidr_blocks = ["::/0"]
  }

  tags = {
    Name = "alb-sg" // used in the lambda to lookup
  }
}

# Create ALB
resource "aws_lb" "puzzlepassportlambda" {
  name               = "my-alb"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb_sg.id]
  subnets            = [aws_subnet.publica.id, aws_subnet.publicb.id]
  ip_address_type    = "dualstack"
}

# Create target group
resource "aws_lb_target_group" "lambda_tg" {
  name        = "lambda-tg"
  target_type = "lambda"
}

# Create listener
resource "aws_lb_listener" "http" {
  load_balancer_arn = aws_lb.puzzlepassportlambda.arn
  port              = "80"
  protocol          = "HTTP"
  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.lambda_tg.arn
  }
}

data "aws_acm_certificate" "tossl" {
  domain   = "puzzlepassport.com"
  types       = ["AMAZON_ISSUED"]
  most_recent = true
}


resource "aws_lb_listener" "https" {
  load_balancer_arn = aws_lb.puzzlepassportlambda.arn
  port              = "443"
  protocol          = "HTTPS"
  certificate_arn   = data.aws_acm_certificate.tossl.arn
  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.lambda_tg.arn
  }
}

# Lambda
# Create IAM role for Lambda
resource "aws_iam_role" "lambda_role" {
  name = "lambda_role" // used in lambda to lookup
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = "sts:AssumeRole"
      Effect = "Allow"
      Principal = {
        Service = "lambda.amazonaws.com"
      }
    }]
  })
}

# Attach policy to the role
resource "aws_iam_role_policy_attachment" "lambda_policy" {
  role       = aws_iam_role.lambda_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaVPCAccessExecutionRole"
}



# Dynamo Lambda configuration
data "aws_caller_identity" "current" {}

resource "aws_iam_policy" "lambda_dynamodb_policy" {
  name = "lambda_dynamodb_policy"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "dynamodb:GetItem",
          "dynamodb:PartiQLSelect",
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
  role       = aws_iam_role.lambda_role.name
  policy_arn = aws_iam_policy.lambda_dynamodb_policy.arn
}

resource "aws_iam_policy" "lambda_s3_policy" {
  name = "lambda_s3_policy"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "s3:PutObject",
        ]
        Resource = "arn:aws:s3:::puzzle-passport-media/*"
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "lambda_s3_attach" {
  role       = aws_iam_role.lambda_role.name
  policy_arn = aws_iam_policy.lambda_s3_policy.arn
}

data "aws_route53_zone" "puzzlepassport" {
    name         = "puzzlepassport.com." 
}

resource "aws_route53_record" "alb_backend_alias_record" {
  zone_id = data.aws_route53_zone.puzzlepassport.zone_id # Replace with your zone ID
  name    = "backend.puzzlepassport.com" # Replace with your name/domain/subdomain
  type    = "A"

  alias {
    name                   = aws_lb.puzzlepassportlambda.dns_name
    zone_id                = aws_lb.puzzlepassportlambda.zone_id
    evaluate_target_health = true
  }
}

resource "aws_route53_record" "vercel_dns_cname_record" {
  zone_id = data.aws_route53_zone.puzzlepassport.zone_id # Replace with your zone ID
  name    = "www.puzzlepassport.com" # Replace with your name/domain/subdomain
  type    = "CNAME"
  ttl = 900 
  records        = [var.vercel_cname_dns]

}

resource "aws_route53_record" "vercel_apex_a_record" {
  zone_id = data.aws_route53_zone.puzzlepassport.zone_id # Replace with your zone ID
  name    = "puzzlepassport.com" # Replace with your name/domain/subdomain
  type    = "A"
  ttl = 300
  records = [var.vercel_ip]

}
