#![allow(unused)]

use std::{env, path::Path};

use anyhow::{bail, Context};
use aws_sdk_s3::{config, types::ByteStream, Client, Credentials, Error, Region};
use dotenv::dotenv;

// -- Constants
const ENV_CRED_KEY_ID: &str = "S3_KEY_ID";
const ENV_CRED_KEY_SECRET: &str = "S3_KEY_SECRET";
const BUCKET_NAME: &str = "rate-n-date-profile-images";
const REGION: &str = "eu-central-1";

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let client = get_aws_client(REGION)?;

    let keys = list_keys(&client, BUCKET_NAME).await.unwrap();
    println!("List:\n{}", keys.join("\n"));

    let path = Path::new("src/main.rs");
    let _res = upload_file(&client, BUCKET_NAME, path).await.unwrap();
    println!("Uploaded file {}", path.display());

    Ok(())
}

async fn list_keys(client: &Client, bucket_name: &str) -> Result<Vec<String>, ()> {
    let req = client.list_objects_v2().prefix("").bucket(bucket_name);

    let result = req.send().await.unwrap();

    let keys = result.contents().unwrap_or_default();
    let keys = keys
        .iter()
        .filter_map(|o| o.key.as_ref())
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    Ok(keys)
}

async fn upload_file(client: &Client, bucket_name: &str, path: &Path) -> Result<(), ()> {
    if !path.exists() {
        println!("Path {} dows not exists", path.display());
    }

    let key = path.to_str().unwrap();

    let body = ByteStream::from_path(&path).await.unwrap();
    let content_type = mime_guess::from_path(&path).first_or_octet_stream().to_string();

    let request = client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(body)
        .content_type(content_type);

    request.send().await;

    Ok(())
}

fn get_aws_client(region: &str) -> Result<Client, Error> {
    let key_id = env::var(ENV_CRED_KEY_ID)
        .context("Missing S3_KEY_ID")
        .unwrap();
    let key_secret = env::var(ENV_CRED_KEY_SECRET)
        .context("Missing S3_KEY_SECRET")
        .unwrap();

    let cred = Credentials::new(key_id, key_secret, None, None, BUCKET_NAME);

    let region = Region::new(region.to_string());
    let conf_builder = config::Builder::new()
        .region(region)
        .credentials_provider(cred);
    let conf = conf_builder.build();

    let client = Client::from_conf(conf);
    Ok(client)
}
