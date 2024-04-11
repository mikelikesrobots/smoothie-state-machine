# Smoothie Ordering App

This is an example project to demonstrate how to construct a Step Functions state machine to send customer orders via AWS IoT Core, and listen for a response to that order.

The use case is a customer ordering a smoothie, triggering the state machine to start up. The state machine finds an available robot from the DynamoDB table, sets its status to WORKING, then transmits a message to it to request the smoothie. If successful, the status is set back to ONLINE; if it times out, the status is set to BROKEN.

## Installing Tools

The tools required are CDK and the Cargo package manager. These can be installed as follows:

To install CDK, first install NodeJS ([more instructions](https://nodejs.org/en/download/package-manager)). AL2 systems can install using `sudo yum install npm`. Then execute:

```bash
sudo npm install -g aws-cdk
```

To install the Rust dependencies, you will require Rust (use [Rustup](https://rustup.rs/)) and Cargo Lambda. Cargo Lambda may be installed in several ways, but the easiest is with Pip. For an AL2 system, this can be accomplished with:

```bash
# Install rust with default options
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Add to current shell
source "$HOME/.cargo/env"
# Install Python's package manager
sudo yum install -y python3-pip
# Install Cargo Lambda
pip3 install cargo-lambda
# Install gcc for linking
sudo yum install -y gcc
```

## Clone the package

If you haven't already, you can clone this package using:

```bash
# Install git if not present
sudo yum install -y git
# Clone package
git clone https://github.com/mikelikesrobots/smoothie-state-machine/
```

## Build and Deploy

First, make sure you have AWS credentials activated on your account. You can then build the package by entering the directory, installing dependencies, and running the deploy command, as follows:

```bash
cd path/to/smoothie-state-machine
npm install
npm run deploy
```

Note that this is not the standard `cdk deploy` command - that's because this deploy contains an extra step of compiling the Rust code to zip files suitable for upload to Lambda.

Once the deployment is ready, it will prompt for permission to continue - enter 'y' and allow it to deploy.

## Add entries to the DynamoDB table

To be able to test the application, you will need at least one entry in the DynamoDB table. The `reset_robot.sh` script can do this automatically for you - just run it as follows:

```bash
./scripts/reset_robot.sh
```

This will create two entries with names `Robot1` and `Robot2`, both set to ONLINE.

## Set up the Mock Robot Script

For a successful execution, a "robot" needs to send a success message before the task times out. The `mock_robot.py` script can help with this.

### Set up device certificate

To set up an IoT certificate for use by the mock robot script, execute the provision script as shown:

```bash
./scripts/provision_robot_cert.sh
```

This will create the `scripts/certs` folder with the required files.

### Install script dependencies

The script is run using Python 3 with `boto3` and `awsiotsdk`. These can be installed using the `pip` package manager:

```bash
pip3 install awsiotsdk boto3
```

### Run Script

You should now be able to execute the mock robot script using:

```bash
python3 scripts/mock_robot.py
```

This will respond to any smoothie requests with success messages after 3 seconds.

## Execute the State Machine

To test the state machine, you can either open the console and execute the state machine with a message such as:

```json
{
    "SmoothieName": "Smoothie Flavour"
}
```

To do this automatically, use the test step function script:

```bash
./scripts/test_stepfunction.sh
```

This will kick off an execution, wait for it to finish, and display the success or error message.

## Teardown

Once your testing is complete, the stack can be removed by executing `cdk destroy` and agreeing with the confirmation prompt. You may also have an IoT certificate registered for the device, which you can delete manually in the console.
