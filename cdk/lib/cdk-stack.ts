import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import * as sqs from 'aws-cdk-lib/aws-sqs';
import * as iam from 'aws-cdk-lib/aws-iam';
import * as cloudwatch from 'aws-cdk-lib/aws-cloudwatch';
import { RustFunction } from "cargo-lambda-cdk";
import * as path from "node:path";
import { Duration } from "aws-cdk-lib";
import { SqsEventSource } from "aws-cdk-lib/aws-lambda-event-sources";

export class CdkStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // The code that defines your stack goes here

    const producerFunction = new RustFunction(this, "Producer", {
      functionName: "producer-function",
      manifestPath: path.join(__dirname, "../../lambdas/starter/Cargo.toml"),
      runtime: "provided.al2023",
    })

    const consumerFunction = new RustFunction(this, "Consumer", {
      functionName: "consumer-function",
      manifestPath: path.join(__dirname, "../../lambdas/consumer/Cargo.toml"),
      runtime: "provided.al2023",
      timeout: Duration.seconds(1)
    });

    if (consumerFunction.timeout) {
      new cloudwatch.Alarm(this, "TimeoutAlarm", {
        metric: consumerFunction.metricDuration(),
        threshold: consumerFunction.timeout.toSeconds(),
        evaluationPeriods: 1,
        datapointsToAlarm: 1,
        comparisonOperator: cloudwatch.ComparisonOperator.GREATER_THAN_THRESHOLD,
        alarmName: "ConsumerFunctionTimeout",
        alarmDescription: "Consumer function timeout"
      })
    }

    const deadLetterQueue = new sqs.Queue(this, "DLQ", {
      queueName: "test-dead-queue.fifo",
      fifo: true,
    });

    const queue = new sqs.Queue(this, 'CdkQueue', {
      queueName: "test-queue.fifo",
      visibilityTimeout: cdk.Duration.seconds(7),
      deadLetterQueue: {
        maxReceiveCount: 3,
        queue: deadLetterQueue
      },
      fifo: true
    });

    consumerFunction.addEventSource(new SqsEventSource(queue, {
      batchSize: 1,
      maxConcurrency: 5,
      reportBatchItemFailures: true
    }))

    queue.grant(producerFunction, "sqs:SendMessage");
    producerFunction.addEnvironment("URL", queue.queueUrl);

    const deadLetterFunction = new RustFunction(this, "DeadLetterConsumer", {
      functionName: "dead-letter-function",
      manifestPath: path.join(__dirname, "../../lambdas/dead-letter-consumer/Cargo.toml"),
      runtime: "provided.al2023",
      timeout: Duration.seconds(5)
    });

    deadLetterFunction.addToRolePolicy(new iam.PolicyStatement({
      actions: ["events:PutEvents"],
      resources: [`arn:aws:events:${this.region}:${this.account}:*`]
    }));

    deadLetterFunction.addEventSource(new SqsEventSource(deadLetterQueue, {
      reportBatchItemFailures: true
    }))
  }
}
