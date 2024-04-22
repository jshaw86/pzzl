#!/bin/bash

AWS_ACCOUNT=$(aws sts get-caller-identity --query Account --output text)
AWS_REGION=${2:-us-east-1}
VERSION=${3:-latest}

aws ecr get-login-password --region $AWS_REGION | docker login --username AWS --password-stdin $AWS_ACCOUNT.dkr.ecr.$AWS_REGION.amazonaws.com

cd pzzl-database

docker build -t pzzl/database .
docker tag pzzl/database:$VERSION $AWS_ACCOUNT.dkr.ecr.$AWS_REGION.amazonaws.com/pzzl/database:$VERSION 
docker push $AWS_ACCOUNT.dkr.ecr.$AWS_REGION.amazonaws.com/pzzl/database:$VERSION 

cd ..

docker build -t pzzl/lambda .
docker tag pzzl/lambda:$VERSION $AWS_ACCOUNT.dkr.ecr.$AWS_REGION.amazonaws.com/pzzl/lambda:$VERSION 
docker push $AWS_ACCOUNT.dkr.ecr.$AWS_REGION.amazonaws.com/pzzl/lambda:$VERSION 
