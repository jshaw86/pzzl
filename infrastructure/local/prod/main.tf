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


