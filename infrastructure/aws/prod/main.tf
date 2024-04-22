resource "aws_ecr_repository" "pzzl_lambda_repository" {
  name = "pzzl/lambda"
}

resource "aws_ecr_repository" "pzzl_database_repository" {
  name = "pzzl/database"
}

resource "aws_vpc" "main" {
  cidr_block = "10.0.0.0/16"
  enable_dns_support = true
  enable_dns_hostnames = true
}

data "aws_availability_zones" "available" {
  state = "available"
}

resource "aws_subnet" "lambda_subnet_primary" {
  vpc_id     = aws_vpc.main.id
  cidr_block = "10.0.1.0/24"

  map_public_ip_on_launch = true
  availability_zone = data.aws_availability_zones.available.names[0]
}

resource "aws_subnet" "lambda_subnet_secondary" {
  vpc_id     = aws_vpc.main.id
  cidr_block = "10.0.2.0/24"

  map_public_ip_on_launch = true
  availability_zone = data.aws_availability_zones.available.names[1]
}

resource "aws_security_group" "lambda_sg" {
  name        = "lambda_sg"
  description = "Security group for Lambda to access RDS"
  vpc_id      = aws_vpc.main.id

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_db_instance" "pzzl_database" {
  identifier = "pzzl-db"

  engine              = "postgres"
  instance_class      = "db.t3.micro"
  allocated_storage   = 20
  db_name               = var.rds_name
  username              = var.rds_username
  password              = var.rds_password
  skip_final_snapshot  = true
  apply_immediately = true


  vpc_security_group_ids = [aws_security_group.rds_sg.id]
  db_subnet_group_name = aws_db_subnet_group.subnet_group.name
}

resource "aws_db_subnet_group" "subnet_group" {
  name       = "my-db-subnet-group"
  subnet_ids = [aws_subnet.lambda_subnet_primary.id, aws_subnet.lambda_subnet_secondary.id]

  tags = {
    Name = "My DB subnet group"
  }
}

resource "aws_security_group" "rds_sg" {
  name        = "rds_sg"
  description = "Security group for RDS instance"
  vpc_id      = aws_vpc.main.id

  ingress {
    from_port   = 3306 // Change this if using a different DB or port
    to_port     = 3306
    protocol    = "tcp"
    security_groups = [aws_security_group.lambda_sg.id]
  }
}

