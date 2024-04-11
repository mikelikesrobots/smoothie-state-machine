import * as cdk from 'aws-cdk-lib';
import { Construct } from "constructs";
import * as ddb from 'aws-cdk-lib/aws-dynamodb';

export class RobotTable extends Construct {
  table: ddb.Table;

  constructor(parent: Construct) {
    super(parent, "RobotTableConstruct");

    this.table = new ddb.Table(this, 'RobotDDBTable', {
      tableName: "StepFunctionTable",
      billingMode: ddb.BillingMode.PAY_PER_REQUEST,
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      partitionKey: {
        name: 'name',
        type: ddb.AttributeType.STRING,
      },
    });
  }
}

export default RobotTable;
