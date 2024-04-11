use aws_sdk_sfn::config::BehaviorVersion;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Request {
    task_token: String,
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

#[tracing::instrument(skip(sfn_client, event), fields(req_id = %event.context.request_id))]
async fn send_success(
    sfn_client: &aws_sdk_sfn::Client,
    event: LambdaEvent<Request>,
) -> Result<Response, Error> {
    tracing::info!("handling an event: {:?}", event.payload);

    let output = json!({"info": "Successfully made smoothie!"}).to_string();

    let request = sfn_client
        .send_task_success()
        .set_output(Some(output))
        .set_task_token(Some(event.payload.task_token.clone()));

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
                    "Successfully sent a task success for task token '{}'",
                    &event.payload.task_token,
                ),
            })
        }
        Err(_err) => {
            Err(Box::new(Response {
                req_id: event.context.request_id,
                body: "The Lambda function encountered an error and no task update was made"
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

    // Initialize the client here to be able to reuse it across
    // different invocations.
    //
    // No extra configuration is needed as long as your Lambda has
    // the necessary permissions attached to its role.
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let sfn_client = aws_sdk_sfn::Client::new(&config);

    lambda_runtime::run(service_fn(|event: LambdaEvent<Request>| async {
        send_success(&sfn_client, event).await
    }))
    .await
}
