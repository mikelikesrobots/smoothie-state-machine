use aws_sdk_dynamodb::config::BehaviorVersion;
use aws_sdk_dynamodb::types::AttributeValue;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "UPPERCASE")]
enum Status {
    Online,
}
impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Serialize::serialize(&self, f)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Request {
    smoothie_name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Response {
    smoothie_name: String,
    robot_name: String,
}

impl std::fmt::Display for Response {
    /// Display the response struct as a JSON string
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_as_json = serde_json::json!(self).to_string();
        write!(f, "{err_as_json}")
    }
}

#[derive(Debug)]
struct ErrorDetails {
    message: String,
}
impl std::fmt::Display for ErrorDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl std::error::Error for ErrorDetails {}

#[tracing::instrument(skip(ddb_client, event), fields(req_id = %event.context.request_id))]
async fn get_available_robot(
    ddb_client: &aws_sdk_dynamodb::Client,
    table_name: &str,
    event: LambdaEvent<Request>,
) -> Result<Response, ErrorDetails> {
    tracing::info!("handling an event: {:?}", event.payload);

    let results = ddb_client
        .scan()
        .table_name(table_name)
        .filter_expression("#st = :stat")
        .expression_attribute_names("#st", "status")
        .expression_attribute_values(":stat" , AttributeValue::S("ONLINE".to_owned()))
        .send()
        .await
        .map_err(|_| ErrorDetails { message: "Failed to get result from table!".to_string() })?;

    let Some([first, ..]) = results.items.as_deref() else {
        tracing::error!("No robot in response: {:#?}", results);
        return Err(ErrorDetails { message: "No available robot".to_string() });
    };

    let Some(name_attr) = first.get("name") else {
        tracing::error!("No status field in row: {:#?}", first);
        return Err(ErrorDetails { message: "No status field in row".to_string() });
    };

    let Ok(name) = name_attr.as_s() else {
        tracing::error!("Unable to render as string: {:#?}", name_attr);
        return Err(ErrorDetails { message: "Could not render name as string".to_string() });
    };

    tracing::info!("Found available robot with name: {}", name);
    Ok(Response {
        smoothie_name: event.payload.smoothie_name,
        robot_name: name.clone(),
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
    let ddb_client = aws_sdk_dynamodb::Client::new(&config);

    lambda_runtime::run(service_fn(|event: LambdaEvent<Request>| async {
        get_available_robot(&ddb_client, &table_name, event).await
    }))
    .await
}
