import { Construct } from "constructs";

import * as ddb from 'aws-cdk-lib/aws-dynamodb';
import * as lambda from 'aws-cdk-lib/aws-lambda';
import * as iam from 'aws-cdk-lib/aws-iam';
import * as step from 'aws-cdk-lib/aws-stepfunctions';

export class Functions extends Construct {

  getAvailableRobotFunction: lambda.Function;
  updateStatusFunction: lambda.Function;
  sendMqttFunction: lambda.Function;
  sendTaskSuccessFunction: lambda.Function;

  constructor(parent: Construct, table: ddb.Table) {
    super(parent, "FunctionsConstruct");

    // Read DDB; get first robot with status ONLINE, or give error code.
    this.getAvailableRobotFunction = new lambda.Function(this, 'AvailableRobotFunction', {
      runtime: lambda.Runtime.PROVIDED_AL2023,
      handler: 'bootstrap',
      code: lambda.Code.fromAsset('./handlers/rust-available-robot/target/lambda/rust-available-robot/bootstrap.zip'),
      architecture: lambda.Architecture.ARM_64,
      environment: {
        "TABLE_NAME": table.tableName,
      },
    });
    if (this.getAvailableRobotFunction.role) {
      table.grantReadData(this.getAvailableRobotFunction.role);
    }

    // Update table with given robot name to given robot status
    this.updateStatusFunction = new lambda.Function(this, 'RobotStatusFunction', {
      runtime: lambda.Runtime.PROVIDED_AL2023,
      handler: 'bootstrap',
      code: lambda.Code.fromAsset('./handlers/rust-update-status/target/lambda/rust-update-status/bootstrap.zip'),
      architecture: lambda.Architecture.ARM_64,
      environment: {
        "TABLE_NAME": table.tableName,
      },
    });
    if (this.updateStatusFunction.role) {
      table.grantWriteData(this.updateStatusFunction.role);
    }

    // Send MQTT message to given robot name with given smoothie order
    this.sendMqttFunction = new lambda.Function(this, 'SendMqttFunction', {
      runtime: lambda.Runtime.PROVIDED_AL2023,
      handler: 'bootstrap',
      code: lambda.Code.fromAsset('./handlers/rust-send-mqtt/target/lambda/rust-send-mqtt/bootstrap.zip'),
      architecture: lambda.Architecture.ARM_64,
    });
    this.sendMqttFunction.role?.addManagedPolicy(iam.ManagedPolicy.fromAwsManagedPolicyName('AWSIoTDataAccess'));

    // Call Step Function task success API with given task token
    this.sendTaskSuccessFunction = new lambda.Function(this, 'SendTaskSuccessFunction', {
      runtime: lambda.Runtime.PROVIDED_AL2023,
      handler: 'bootstrap',
      code: lambda.Code.fromAsset('./handlers/rust-send-task-success/target/lambda/rust-send-task-success/bootstrap.zip'),
      architecture: lambda.Architecture.ARM_64,
    });
  }

  updateTaskSuccessRole(machine: step.StateMachine) {
    const sendTaskSuccessPolicy = new iam.Policy(this, "SendTaskSuccessPolicy", {
      statements: [
        new iam.PolicyStatement({
          actions: ["states:SendTaskSuccess"],
          resources: [machine.stateMachineArn],
        })
      ]
    });
    this.sendTaskSuccessFunction.role?.attachInlinePolicy(sendTaskSuccessPolicy);
  }
}

export default Functions;
