#!/bin/bash
#

REPOSITORY_URI=$(aws ecr describe-repositories --repository-name pzzl/database --query "repositories[0].repositoryUri" --output text)
DATABASE_URI=$(aws rds describe-db-instances --db-instance-identifier pzzl-db --query "DBInstances[0].Endpoint.Address" --output text)
DATABASE_USER=$1
DATABASE_PASSWORD=$2
IMAGE_VERSION=${3:-latest}
AWS_ACCESS_KEY_ID=$(aws configure get  aws_access_key_id)
AWS_SECRET_ACCESS_KEY=$(aws configure get aws_secret_access_key)

# Determine OS and calculate future time accordingly
if [[ "$(uname)" == "Linux" ]]; then
    # Linux date command with -d option
    SCHEDULE_TIME=$(date -u -d "2 minutes" '+%M %H %d %m ? %Y')
elif [[ "$(uname)" == "Darwin" ]]; then
    # macOS date command with -v option
    SCHEDULE_TIME=$(date -u -v +2M '+%M %H %d %m ? %Y')
else
    echo "Unsupported OS"
    exit 1
fi

set -x

terraform -chdir=infrastructure/aws/prod/database plan -var access_key="$AWS_ACCESS_KEY_ID" -var secret_key="$AWS_SECRET_ACCESS_KEY" \
    -var repository_url="$REPOSITORY_URI" -var image_version="$IMAGE_VERSION" -var database_url="$DATABASE_URI" \
    -var database_user="$DATABASE_USER" -var database_password="$DATABASE_PASSWORD" -var schedule_time="cron($SCHEDULE_TIME)"


