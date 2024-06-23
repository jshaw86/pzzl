data "aws_route_tables" "main_vpc_route_tables" {
  vpc_id = aws_vpc.main.id 
}

resource "aws_vpc_endpoint" "dynamodb" {
  vpc_id       = aws_vpc.main.id
  service_name = "com.amazonaws.${var.region}.dynamodb"
  vpc_endpoint_type = "Gateway"

  route_table_ids = data.aws_route_tables.main_vpc_route_tables.ids 

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
