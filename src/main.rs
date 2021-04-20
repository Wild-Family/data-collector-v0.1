extern crate dotenv;
#[macro_use]
extern crate dotenv_codegen;
extern crate hmac;
extern crate sha2;
extern crate url;

use dotenv::dotenv;
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;
use std::collections::HashMap;
use std::env;
use std::time::SystemTime;
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get ENV
    let FTX_API_KEY = dotenv!("FTX_API_KEY");
    let FTX_API_SECRET = dotenv!("FTX_API_SECRET");

    println!("FTX_API_KEY: {}", FTX_API_KEY);
    println!("FTX_API_SECRET: {}", FTX_API_SECRET);

    // Create HMAC
    let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_millis().to_string(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };
    let method = "GET";
    let endpoint = Url::parse("https://ftx.com/api/markets").expect("Cannot parse url");
    let mut hmac = Hmac::<Sha256>::new_varkey(FTX_API_SECRET.as_bytes())
        .expect("HMAC can take key of any size");
    let sign_payload = format!("{}{}{}", timestamp, method, endpoint.path());
    hmac.update(sign_payload.as_bytes());
    let hmac_slice = hmac.finalize().into_bytes();
    let hmac_result = hmac_slice.iter().map(|n| format!("{:02x}", n)).collect::<String>();

    println!("sign_payload: {}", sign_payload);

    // Make a request
    let res = reqwest::Client::new()
        .get(endpoint)
        .header("FTX-KEY", FTX_API_KEY)
        .header("FTX-SIGN", hmac_result)
        .header("FTX-TS", timestamp)
        .send()
        .await?;
    println!("{:?}", res);
    println!("{:?}", res.text().await?);
    Ok(())
}