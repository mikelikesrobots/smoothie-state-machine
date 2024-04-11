import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';

import RobotTable from './components/robot_table';
import Functions from './components/functions';
import IoTRules from './components/iot_rules';
import SmoothieOrderHandler from './components/smoothie_order_handler';

export class CdkStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const robotTable = new RobotTable(this);
    const lambdaFunctions = new Functions(this, robotTable.table);
    const rules = new IoTRules(this, lambdaFunctions);
    const smoothieOrderHandler = new SmoothieOrderHandler(this, lambdaFunctions);

    lambdaFunctions.updateTaskSuccessRole(smoothieOrderHandler.orderStateMachine);
  }
}
