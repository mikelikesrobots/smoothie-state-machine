use aws_sdk_iotdataplane::config::BehaviorVersion;
use aws_smithy_types::Blob;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Request {
    task_token: String,
    robot_name: String,
    smoothie_name: String,
}

#[derive(Debug, Serialize)]
struct Order<'a> {
    task_token: &'a String,
    smoothie: &'a String,
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

#[tracing::instrument(skip(iot_client, event), fields(req_id = %event.context.request_id))]
async fn send_smoothie_order(
    iot_client: &aws_sdk_iotdataplane::Client,
    table_name: &str,
    event: LambdaEvent<Request>,
) -> Result<Response, Error> {
    tracing::info!("handling an event: {:?}", event.payload);

    let topic = format!("robots/{}/order", event.payload.robot_name);
    let payload = {
        let payload_raw = Order { task_token: &event.payload.task_token, smoothie: &event.payload.smoothie_name};
        let Ok(payload) = serde_json::to_string(&payload_raw) else {
            tracing::error!("Not able to serialize smoothie order: {:#?}", payload_raw);
            return Err(Box::new(Response {
                req_id: event.context.request_id,
                body: "".to_owned()
            }));
        };
        payload
    };

    iot_client.publish()
        .topic(topic)
        .payload(Blob::new(payload))
        .send()
        .await?;

    Ok(Response {
        req_id: event.context.request_id,
        body: format!(
            "Successfully sent smoothie {} to robot {}",
            event.payload.smoothie_name, event.payload.robot_name
        ),
    })
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
    let iot_client = aws_sdk_iotdataplane::Client::new(&config);

    lambda_runtime::run(service_fn(|event: LambdaEvent<Request>| async {
        send_smoothie_order(&iot_client, &table_name, event).await
    }))
    .await
}
