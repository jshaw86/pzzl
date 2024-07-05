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
curl -v -H "content-type:application/json" -X PUT -d '{"num_pieces": 1, "title":"puzzle title", "name":"thing", "url":"puzzle media url", "users":[{"name":"pzzl user name", "email":"some puzzle user email"}], "lat":12.21, "lng":83.21, "completion_time":86868494, "stamps":[{"users":[{"email":"thing email2", "name":"some user name2"}], "name":"stamp name", "missing_pieces": 4, "puzzlers": 2,"completion_time":1232131, "urls":["dfsfsdfsd"], "lat":63.31, "lng":32.80 }]}' http://localhost:8089/puzzles
`

- Get puzzle 
`curl -v -H "content-type:application/json" -X GET http://localhost:8089/puzzles/<puzzle_id>
` 
- Add Stamp 
`
curl -v -H "content-type:application/json" -X PUT -d '[{"users":[{"email":"thing email2", "name":"some user name2"}], "name":"stamp name", "missing_pieces": 4, "puzzlers": 2,"completion_time":1232131, "media":"dfsfsdfsd", "lat":63.31, "lng":32.80, "urls":["dfssfdsfssd"] }]' http://localhost:8089/puzzles/<puzzle_id>/stamps
`
- Media presigned
`
curl -v http://localhost:8089/media/someprefix
`
## aws cli
- table scan `aws --endpoint-url=http://localhost:4566 dynamodb scan --table-name puzzles_users`


## python simple server with index.html
from the project root `python -m http.server 8000`

