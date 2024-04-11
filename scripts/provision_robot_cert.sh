#!/bin/bash

CERTS_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )/certs
POLICY_NAME="IoTFullAccess"

if ! test -f $CERTS_DIR; then
    mkdir -p $CERTS_DIR
fi

if test -f $CERTS_DIR/mock_robot_cert.pem; then
    echo "Mock robot certificate exists - exiting."
    exit 0
fi

# Create policy if it doesn't exist
aws iot get-policy --policy-name $POLICY_NAME > /dev/null 2>&1
if [ $? != 0 ]; then
    set -e
    echo "Policy $POLICY_NAME doesn't exist - creating..."
    aws iot create-policy --policy-name $POLICY_NAME --policy-document '{"Version": "2012-10-17", "Statement": [{"Effect": "Allow", "Action": "*", "Resource": "*"}]}'
    echo "Created successfully"
    set +e
fi

set -e

echo "Creating certificate..."
CERT_ARN=$(aws iot create-keys-and-certificate \
    --set-as-active \
    --certificate-pem-outfile $CERTS_DIR/mock_robot_cert.pem \
    --public-key-outfile $CERTS_DIR/mock_robot_pub.key \
    --private-key-outfile $CERTS_DIR/mock_robot_priv.key \
    --output text \
    --query certificateArn
)

echo "Created successfully. Attaching policy..."
aws iot attach-policy \
    --target $CERT_ARN \
    --policy-name $POLICY_NAME

echo "Done."
