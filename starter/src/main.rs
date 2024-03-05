use aws_config::BehaviorVersion;
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use aws_sdk_sqs::{ Client};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// This is a made-up example. Requests come into the runtime as unicode
/// strings in json format, which can map to any structure that implements `serde::Deserialize`
/// The runtime pays no attention to the contents of the request payload.
#[derive(Deserialize)]
struct Request {
    command: String,
}

/// This is a made-up example of what a response structure may look like.
/// There is no restriction on what it can be. The runtime requires responses
/// to be serialized into json. The runtime pays no attention
/// to the contents of the response payload.
#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

#[derive(Serialize)]
struct Message {
    id: usize,
    message: String
}

async fn send_queue(client: &Client, url: &String, id: usize) -> Result<(), Error> {

    let message = Message {
        id,
        message: format!("message from: {:?}", id)
    };

    let uuid = Uuid::new_v4();

    client
        .send_message()
        .queue_url(url)
        .message_body(serde_json::to_string(&message)?)
        .message_deduplication_id(format!("{}", uuid))
        .message_group_id(format!("{}", uuid))
        .send()
        .await?;

    Ok(())
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<Request>) -> Result<Response, Error> {
    // Extract some useful info from the request
    let command = event.payload.command;

    let url = std::env::var("URL").unwrap();

    let config = aws_config::defaults(BehaviorVersion::latest()).load().await;

    let client = Client::new(&config);

    for i in 0..20 {
        send_queue(&client, &url, i as usize).await.expect("TODO: panic message");
    }

    // Prepare the response
    let resp = Response {
        req_id: event.context.request_id,
        msg: format!("Command {}.", command),
    };

    // Return `Response` (it will be serialized to JSON automatically by the runtime)
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
