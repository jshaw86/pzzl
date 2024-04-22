# pzzl

## Setup
- https://brew.sh/
- `brew install rust postgresql@16`
- https://www.docker.com/products/docker-desktop/
- `export PATH="/opt/homebrew/bin:$PATH"`

## Run database 
`docker run --name pzzl -p 127.0.0.1:5432:5432  -e POSTGRES_PASSWORD=mysecretpassword -e POSTGRES_USER=postgres -e POSTGRES_DB=pzzl -d postgres`

Run database migration
- `cd pzzl/pzzl-database`
- `cargo run` 

Run server  
- `cd pzzl/pzzl-server` 
- `RUST_BACKTRACE=1 cargo run`

Restart database
- `docker ps`
- `docker kill <container id>`
- `docker rm pzzl`

Connect to database
`psql  -h localhost -p 5432 -U postgres -W`

