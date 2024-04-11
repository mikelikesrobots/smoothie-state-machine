import * as cdk from 'aws-cdk-lib';
import { Construct } from "constructs";

import * as step from 'aws-cdk-lib/aws-stepfunctions';
import * as steptasks from 'aws-cdk-lib/aws-stepfunctions-tasks';

import Functions from './functions';

export class SmoothieOrderHandler extends Construct {

  orderStateMachine: step.StateMachine;

  constructor(parent: Construct, functions: Functions) {
    super(parent, "SmoothieOrderHandlerConstruct");

    const getAvailableRobot = new steptasks.LambdaInvoke(this, 'GetRobot', {
      lambdaFunction: functions.getAvailableRobotFunction,
      outputPath: "$.Payload",
    });

    const setRobotWorking = new steptasks.LambdaInvoke(this, 'SetRobotWorking', {
      lambdaFunction: functions.updateStatusFunction,
      payload: step.TaskInput.fromObject({
        "RobotName.$": "$.RobotName",
        "Status": "WORKING",
      }),
      resultPath: step.JsonPath.DISCARD,
    });

    const tellRobotOrder = new steptasks.LambdaInvoke(this, 'TellRobotOrder', {
      lambdaFunction: functions.sendMqttFunction,
      integrationPattern: step.IntegrationPattern.WAIT_FOR_TASK_TOKEN,
      taskTimeout: step.Timeout.duration(cdk.Duration.seconds(10)),
      payload: step.TaskInput.fromObject({
        "TaskToken": step.JsonPath.taskToken,
        "RobotName.$": "$.RobotName",
        "SmoothieName.$": "$.SmoothieName",
      }),
      resultPath: step.JsonPath.DISCARD,
    });

    const setRobotFinished = new steptasks.LambdaInvoke(this, 'SetRobotFinished', {
      lambdaFunction: functions.updateStatusFunction,
      payload: step.TaskInput.fromObject({
        "RobotName.$": "$.RobotName",
        "Status": "ONLINE",
      }),
      resultPath: step.JsonPath.DISCARD,
    });

    const setRobotBroken = new steptasks.LambdaInvoke(this, 'SetRobotBroken', {
      lambdaFunction: functions.updateStatusFunction,
      payload: step.TaskInput.fromObject({
        "RobotName.$": "$.RobotName",
        "Status": "BROKEN",
      }),
      resultPath: step.JsonPath.DISCARD,
    });

    const finishSuccess = new step.Succeed(this, 'StateMachineFinish', { comment: "Robot made smoothie successfully!" });
    const finishFailure = new step.Fail(this, 'StateMachineFailed', { comment: "Timed out waiting for robot to make smoothie!" });

    const orderDef =
      getAvailableRobot
        .next(setRobotWorking)
        .next(tellRobotOrder
          .addCatch(setRobotBroken.next(finishFailure),
            {
              errors: [step.Errors.TIMEOUT],
              resultPath: step.JsonPath.DISCARD,
            })
        )
        .next(setRobotFinished)
        .next(finishSuccess);

    this.orderStateMachine = new step.StateMachine(this, 'SmoothieOrderHandler', {
      definitionBody: step.DefinitionBody.fromChainable(orderDef),
      timeout: cdk.Duration.minutes(15),
      comment: "Handles sending smoothie orders to robots",
    });
  }
}

export default SmoothieOrderHandler;
