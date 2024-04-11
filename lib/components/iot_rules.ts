import { Construct } from "constructs";

import * as iam from 'aws-cdk-lib/aws-iam';
import * as iot from 'aws-cdk-lib/aws-iot';

import Functions from './functions';

export class RobotIoTRules extends Construct {
  sendTaskSuccessRule: iot.CfnTopicRule;

  constructor(parent: Construct, functions: Functions) {
    super(parent, "RobotIoTRulesConstruct");

    const lambdaAction: iot.CfnTopicRule.LambdaActionProperty = {
      functionArn: functions.sendTaskSuccessFunction.functionArn,
    };
    const action: iot.CfnTopicRule.ActionProperty = {
      lambda: lambdaAction,
    };
    this.sendTaskSuccessRule = new iot.CfnTopicRule(this, `FinishedSmoothieRule`, {
      topicRulePayload: {
        sql: `SELECT * FROM 'robots/+/success'`,
        actions: [action],
      },
      ruleName: "FinishedSmoothie",
    });
    functions.sendTaskSuccessFunction.addPermission(`TaskSuccessLambdaPermission`, {
      principal: new iam.ServicePrincipal("iot.amazonaws.com"),
      sourceArn: this.sendTaskSuccessRule.attrArn,
      action: "lambda:InvokeFunction",
    });
  }
}

export default RobotIoTRules;
