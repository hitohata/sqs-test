use aws_lambda_events::event::sqs::SqsEvent;
use aws_lambda_events::sqs::{BatchItemFailure, SqsBatchResponse};
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use rand::Rng;

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<SqsEvent>) -> Result<SqsBatchResponse, Error> {
    // Extract some useful information from the request

    println!("{:?}", event.payload);

    std::thread::sleep(std::time::Duration::from_secs(2));

    let mut batch_item_failures = Vec::new();
    let mut rng = rand::thread_rng();

    if event.payload.records.len() > 1 {
        for item in &event.payload.records {
            batch_item_failures.push(BatchItemFailure {
                item_identifier: item.clone().message_id.unwrap()
            })
        }
    }

    for item in &event.payload.records {
        match &item.message_id {
            Some(id) => {
                let rand_val = rng.gen_range(0..10);
                println!("{:?}", item.body);
                if rand_val >= 9 {
                } else {
                    println!("[ERROR]: {:?}", id.to_owned());
                    batch_item_failures.push(BatchItemFailure {
                        item_identifier: id.to_owned()
                    })
                }
            },
            None =>  {
                panic!("ID!")
            }
        }
    }

    println!("{:?}", batch_item_failures);

    Ok(SqsBatchResponse {
        batch_item_failures
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
