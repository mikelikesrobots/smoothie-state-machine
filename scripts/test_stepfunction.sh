#!/bin/bash
set +e
ARN=$(aws stepfunctions list-state-machines --query stateMachines[0].stateMachineArn --output text)
ID=$(aws stepfunctions start-execution --state-machine-arn $ARN --input '{"SmoothieName":"Strawberry Smoothie"}' --query executionArn --output text)

while [ $(aws stepfunctions describe-execution --execution-arn $ID --output text --query status) = "RUNNING" ]
do
   echo "Waiting for stepfunction to finish..."
   sleep 1
done

aws stepfunctions describe-execution --execution-arn $ID
