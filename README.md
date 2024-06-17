# pzzl

## Setup
- https://brew.sh/
- `brew install rust localstack`
- https://www.docker.com/products/docker-desktop/
- `export PATH="/opt/homebrew/bin:$PATH"`

## Run database 
- `DYNAMODB_SHARE_DB=1 localstack start`

## Create database table
- `cd pzzl/infrastructure/local/prod`
- `terraform apply -auto-approve`

## Run server  
- `cd pzzl/pzzl-lambda` 
- `DYNAMO_ENDPOINT=http://localhost:4566 cargo run`

## Useful Curls
- Insert first puzzle/user `curl -v -H "content-type:application/json" -X PUT -d '{"puzzle_id":"thing4", "name":"namething2", "media":"some media2", "users":[{"user":{"user_id": "3", "email":"thing email2", "name":"some user name2"}, "lat":63.31, "lng":32.80}]}' http://localhost:8089/puzzles
`
- Get puzzle ` curl -v -H "content-type:application/json" -X GET http://localhost:8089/puzzles/thing2
` 
- Add User ` curl -v -H "content-type:application/json" -X PUT -d '{"user":{"user_id": "4", "email":"thing email4", "name":"some user name4"}, "lat":63.31, "lng":32.80}' http://localhost:8089/puzzles/thing4/users
`

## aws cli
- table scan `aws --endpoint-url=http://localhost:4566 dynamodb scan --table-name puzzles_users`
