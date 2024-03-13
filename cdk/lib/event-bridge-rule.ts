import {aws_events_targets, Stack, StackProps} from "aws-cdk-lib";
import {Construct} from "constructs";
import * as events from "aws-cdk-lib/aws-events";
import {RustFunction} from "cargo-lambda-cdk";
import * as path from "path";
import * as ssm from "aws-cdk-lib/aws-ssm";

export class SQSBridgeRule extends Stack {
    constructor(scope: Construct, id: string, props: StackProps) {
        super(scope, id, props);

        const uri = ssm.StringParameter.valueForStringParameter(this, "/sqs-test/discord-hook-url");

        const discordClient = new RustFunction(this, "DiscordClient", {
            functionName: "discord-client-function",
            manifestPath: path.join(__dirname, "../../lambdas/discord-client/Cargo.toml"),
            runtime: "provided.al2023",
        });

        discordClient.addEnvironment("URI", uri);

        new events.Rule(this, "SQSBridgeRule", {
            eventPattern: {
                source: ["notification.discord.dead-letter"],
                detailType: ["Discord"],
            },
            targets: [new aws_events_targets.LambdaFunction(discordClient)]
        });
    }
}