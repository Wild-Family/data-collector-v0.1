use hmac::{Hmac, Mac, NewMac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use std::error::Error;
use std::fmt::Display;
use std::time::SystemTime;
use url::Url;
use std::collections::HashMap;


#[derive(Debug)]
pub enum ExchangeClientError {
    Description(String),
}

impl Display for ExchangeClientError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        println!("{:#?}", self);
        return Ok(());
    }
}

impl Error for ExchangeClientError {}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct FtxMarketData {
    name                    : String,
    ask                     : Option<f64>,
    bid                     : Option<f64>,
    last                    : Option<f64>,
    baseCurrency            : Option<String>,
    change1h                : Option<f64>,
    change24h               : Option<f64>,
    changeBod               : Option<f64>,
    enabled                 : Option<bool>,
    highLeverageFeeExempt   : Option<bool>,
    minProvideSize          : Option<f64>,
    postOnly                : Option<bool>,
    price                   : Option<f64>,
    priceIncrement          : Option<f64>,
    quoteCurrency           : Option<String>,
    quoteVolume24h          : Option<f64>,
    restricted              : Option<bool>,
    sizeIncrement           : Option<f64>,
    r#type                  : Option<String>,
    underlying              : Option<String>,
    volumeUsd24h            : Option<f64>,
}

pub struct FtxClient {
    api_key: String,
    api_secret: String,
}

impl FtxClient {
    pub fn new(api_key: String, api_secret: String) -> Self {
        FtxClient {
            api_key: api_key,
            api_secret: api_secret,
        }
    }

    pub async fn get_markets(self) -> Result<HashMap<String, FtxMarketData>, Box<dyn std::error::Error>> {
        // Create HMAC
        let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_millis().to_string(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };
        let method = "GET";
        let endpoint = Url::parse("https://ftx.com/api/markets").expect("Cannot parse url");
        let mut hmac = Hmac::<Sha256>::new_varkey(self.api_secret.as_bytes())
            .expect("HMAC can take key of any size");
        let sign_payload = format!("{}{}{}", timestamp, method, endpoint.path());
        hmac.update(sign_payload.as_bytes());
        let hmac_slice = hmac.finalize().into_bytes();
        let hmac_result = hmac_slice
            .iter()
            .map(|n| format!("{:02x}", n))
            .collect::<String>();

        println!("sign_payload: {}", sign_payload);

        // Make a request
        let response = reqwest::Client::new()
            .get(endpoint)
            .header("FTX-KEY", self.api_key)
            .header("FTX-SIGN", hmac_result)
            .header("FTX-TS", timestamp)
            .send()
            .await?;

        let value = response.json::<Value>().await?;
        let result = &value["result"];
        let market_data_vals = match result.as_array() {
            Some(value) => value.clone(),
            None => {
                return Err(Box::new(ExchangeClientError::Description(
                    "cannot retrieve object".to_string(),
                )))
            }
        };

        let mut market_datas = HashMap::<String, FtxMarketData>::new();
        for market_data_val in market_data_vals {
            let market_data: FtxMarketData = match serde_json::from_value(market_data_val) {
                Ok(data) => data,
                Err(err) => {
                    println!("{:#?}", err);
                    panic!("cannot parse market data from value");
                }
            };
            market_datas.insert(market_data.name.clone(), market_data);
        }

        Ok(market_datas)
    }
}
