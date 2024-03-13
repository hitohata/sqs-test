use aws_lambda_events::event::sqs::SqsEvent;use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use aws_config::BehaviorVersion;
use aws_sdk_eventbridge::Client;
use aws_sdk_eventbridge::types::PutEventsRequestEntry;
use serde::Serialize;
use serde_json::json;


/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(event: LambdaEvent<SqsEvent>) -> Result<(), Error> {
    // Extract some useful information from the request

    println!("{:?}", event);

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = Client::new(&config);

    let payloads = event.payload.records.iter().map(|el| {
            EventDetail::new(
                String::from("Error"),
                String::from(match &el.message_id { Some(id) => id.to_owned(), None => String::from("none") } )
            ).to_json_string()
        })
        .map(|payload| {
            PutEventsRequestEntry::builder()
                .set_detail(Some(payload))
                .source("notification.discord.dead-letter")
                .detail_type(String::from("Discord"))
                .build()
        })
        .collect::<Vec<PutEventsRequestEntry>>();

    client
        .put_events()
        .set_entries(Some(payloads))
        .send()
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}

#[derive(Serialize)]
struct EventDetail {
    status: String,
    content: String
}

impl EventDetail {
    fn new(status: String, content: String) -> Self {
        Self {
            status,
            content
        }
    }

    fn to_json_string(&self) -> String {
        json!({
            "status": self.status.to_owned(),
            "content": self.content.to_owned()
        }).to_string()
    }
}