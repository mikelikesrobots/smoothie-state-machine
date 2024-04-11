#!/bin/bash
aws dynamodb update-item --table-name StepFunctionTable --key '{"name": {"S": "Robot1"}}' --attribute-updates '{"status": {"Value": {"S": "ONLINE"},"Action": "PUT"}}'
aws dynamodb update-item --table-name StepFunctionTable --key '{"name": {"S": "Robot2"}}' --attribute-updates '{"status": {"Value": {"S": "ONLINE"},"Action": "PUT"}}'
