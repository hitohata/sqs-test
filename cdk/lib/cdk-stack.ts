import * as cdk from 'aws-cdk-lib';
import { Construct } from 'constructs';
import * as sqs from 'aws-cdk-lib/aws-sqs';
import { RustFunction } from "cargo-lambda-cdk";
import * as path from "node:path";
import {Duration} from "aws-cdk-lib";
import {SqsEventSource} from "aws-cdk-lib/aws-lambda-event-sources";

export class CdkStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // The code that defines your stack goes here

    const producerFunction = new RustFunction(this, "Producer", {
      functionName: "producer-function",
      manifestPath: path.join(__dirname, "../../starter/Cargo.toml"),
      runtime: "provided.al2023",
    })

    const consumerFunction = new RustFunction(this, "Consumer", {
      functionName: "consumer-function",
      manifestPath: path.join(__dirname, "../../lambda/Cargo.toml"),
      runtime: "provided.al2023",
      timeout: Duration.seconds(5)
    });

    const queue = new sqs.Queue(this, 'CdkQueue', {
      queueName: "test-queue.fifo",
      visibilityTimeout: cdk.Duration.seconds(7),
      deadLetterQueue: {
        maxReceiveCount: 3,
        queue: new sqs.Queue(this, "DLQ", {
          queueName: "test-dead-queue.fifo",
          fifo: true
        })
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
  }

}
