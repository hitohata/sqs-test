use aws_lambda_events::event::sqs::SqsEvent;
use aws_lambda_events::sqs::{BatchItemFailure, SqsBatchResponse};
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};


/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<SqsEvent>) -> Result<SqsBatchResponse, Error> {
    // Extract some useful information from the request

    let mut batch_item_failures = Vec::new();

    if event.payload.records.len() > 1 {
        for item in &event.payload.records {
            batch_item_failures.push(BatchItemFailure {
                item_identifier: item.clone().message_id.unwrap()
            })
        }
    }

    println!("{:?}", event.payload.records[0].body);

    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    Ok(SqsBatchResponse {
        batch_item_failures
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
