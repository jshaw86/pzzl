#!/bin/bash

set -x

AWS_PROFILE=default
AWS_ACCOUNT=$(aws sts get-caller-identity --query Account --output text)
VERSION=${1:-latest}

cargo clean

aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin $AWS_ACCOUNT.dkr.ecr.us-east-1.amazonaws.com

docker build -t pzzl/database:$VERSION . --progress=plain
docker tag pzzl/database:$VERSION $AWS_ACCOUNT.dkr.ecr.us-east-1.amazonaws.com/pzzl/database:$VERSION
docker push $AWS_ACCOUNT.dkr.ecr.us-east-1.amazonaws.com/pzzl/database:$VERSION




