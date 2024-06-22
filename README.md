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
- `CORS_ORIGIN=http://localhost:8000 DYNAMO_ENDPOINT=http://localhost:4566 cargo run`

## Useful Curls
- Insert first puzzle/user 
`
curl -v -H "content-type:application/json" -X PUT -d '{"num_pieces": 1, "name":"namething2", "media":"some media2", "stamps":[{"user":{"email":"thing email2", "name":"some user name2", "date": "2019-10-24T00:00:00", "owned":true}, "name":"stamp name", "missing_pieces": 4, "puzzlers": 2,"completed_time":1232131, "media":"dfsfsdfsd", "lat":63.31, "lng":32.80 }]}' http://localhost:8089/puzzles
`

- Get puzzle 
`curl -v -H "content-type:application/json" -X GET http://localhost:8089/puzzles/thing2
` 
- Add User 
`curl -v -H "content-type:application/json" -X PUT \
-d '[{"user":{"user_id": "3", "email":"thing email2", "name^":"some user name2", "date*": "2019-10-24T00:00:00", "owned*":true}, \
             "name^": "stamp name", "missing pieces": 4, "completed_time^":1232131, "puzzlers": 2, "media":"dfsfsdfsd", "lat":63.31, "lng":32.80, "date": "2019-10-24T00:00:00"}'] \
http://localhost:8089/puzzles/thing4/stamps`

## aws cli
- table scan `aws --endpoint-url=http://localhost:4566 dynamodb scan --table-name puzzles_users`


## python simple server with index.html
from the project root `python -m http.server 8000`

