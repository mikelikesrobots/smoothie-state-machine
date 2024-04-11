use aws_sdk_dynamodb::config::BehaviorVersion;
use aws_sdk_dynamodb::types::{AttributeValue, AttributeValueUpdate};
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "UPPERCASE")]
enum Status {
    Online,
    Working,
    Broken,
}
impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Serialize::serialize(&self, f)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Request {
    robot_name: String,
    status: Status,
}

#[derive(Debug, Serialize)]
struct Response {
    req_id: String,
    body: String,
}

impl std::fmt::Display for Response {
    /// Display the response struct as a JSON string
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_as_json = serde_json::json!(self).to_string();
        write!(f, "{err_as_json}")
    }
}

impl std::error::Error for Response {}

#[tracing::instrument(skip(ddb_client, event), fields(req_id = %event.context.request_id))]
async fn update_status(
    ddb_client: &aws_sdk_dynamodb::Client,
    table_name: &str,
    event: LambdaEvent<Request>,
) -> Result<Response, Error> {
    tracing::info!("handling an event: {:?}", event.payload);
    let status_str = format!("{}", &event.payload.status);
    let status = AttributeValueUpdate::builder().value(AttributeValue::S(status_str)).build();
    let name = AttributeValue::S(event.payload.robot_name.clone());
    let request = ddb_client
        .update_item()
        .table_name(table_name)
        .key("name", name)
        .attribute_updates("status", status);
    tracing::info!("Executing request [{request:?}]...");

    let response = request
        .send()
        .await;
    tracing::info!("Got response: {:#?}", response);

    match response {
        Ok(_) => {
            // Return `Response` (it will be serialized to JSON automatically by the runtime)
            Ok(Response {
                req_id: event.context.request_id,
                body: format!(
                    "the Lambda function has successfully updated your robot '{}' with status '{}'",
                    &event.payload.robot_name, &event.payload.status,
                ),
            })
        }
        Err(_err) => {
            Err(Box::new(Response {
                req_id: event.context.request_id,
                body: "The Lambda function encountered an error and your status was not updated"
                    .to_owned(),
            }))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let table_name = std::env::var("TABLE_NAME")
        .expect("A TABLE_NAME must be set in this app's Lambda environment variables.");

    // Initialize the client here to be able to reuse it across
    // different invocations.
    //
    // No extra configuration is needed as long as your Lambda has
    // the necessary permissions attached to its role.
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let ddb_client = aws_sdk_dynamodb::Client::new(&config);

    lambda_runtime::run(service_fn(|event: LambdaEvent<Request>| async {
        update_status(&ddb_client, &table_name, event).await
    }))
    .await
}
