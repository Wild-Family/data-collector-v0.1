use chrono::{DateTime, Local};
use hmac::{Hmac, Mac, NewMac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use url::Url;

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
    name: String,
    ask: Option<f64>,
    bid: Option<f64>,
    last: Option<f64>,
    baseCurrency: Option<String>,
    change1h: Option<f64>,
    change24h: Option<f64>,
    changeBod: Option<f64>,
    enabled: Option<bool>,
    highLeverageFeeExempt: Option<bool>,
    minProvideSize: Option<f64>,
    postOnly: Option<bool>,
    price: Option<f64>,
    priceIncrement: Option<f64>,
    quoteCurrency: Option<String>,
    quoteVolume24h: Option<f64>,
    restricted: Option<bool>,
    sizeIncrement: Option<f64>,
    r#type: Option<String>,
    underlying: Option<String>,
    volumeUsd24h: Option<f64>,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct FtxTradeData {
    close: f64,
    high: f64,
    low: f64,
    open: f64,
    volume: f64,
    startTime: DateTime<Local>,
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

    pub async fn generate_sign(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(String::new())
    }

    pub async fn get_trades(
        &self,
        market_name: &str,
        resolution: u32,
        limit: u32,
        start_time: Option<DateTime<Local>>,
        end_time: Option<DateTime<Local>>,
    ) -> Result<Vec<FtxTradeData>, Box<dyn std::error::Error>> {
        // Create HMAC
        let timestamp = Local::now().timestamp_subsec_micros();
        let method = "GET";
        let endpoint = Url::parse(&format!(
            "https://ftx.com/api/markets/{}/candles",
            market_name
        ))
        .expect("Cannot parse url");
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

        let mut query: Vec<(String, String)> = vec![
            ("resolution".to_string(), resolution.to_string()),
            ("limit".to_string(), limit.to_string()),
        ];

        if let Some(start_time) = start_time {
            query.push(("start_time".to_string(), start_time.timestamp_subsec_millis().to_string()))
        }
        if let Some(end_time) = end_time {
            query.push(("end_time".to_string(), end_time.timestamp_subsec_millis().to_string()))
        }

        // Make a request
        let response = reqwest::Client::new()
            .get(endpoint)
            .query(&query)
            .header("FTX-KEY", &self.api_key)
            .header("FTX-SIGN", &hmac_result)
            .header("FTX-TS", timestamp)
            .send()
            .await?;
        
        println!("{:#?}", response.status());

        let value = response.json::<Value>().await?;
        let result = &value["result"];
        let trades_vals = match result.as_array() {
            Some(value) => value.clone(),
            None => {
                return Err(Box::new(ExchangeClientError::Description(
                    "cannot retrieve object".to_string(),
                )))
            }
        };

        let mut trades = Vec::new();
        for trade in trades_vals {
            let trade = match serde_json::from_value(trade) {
                Ok(data) => data,
                Err(err) => {
                    println!("{:#?}", err);
                    panic!("cannot parse market data from value");
                }
            };
            trades.push(trade);
        }

        Ok(trades)
    }

    pub async fn get_markets(
        self,
    ) -> Result<HashMap<String, FtxMarketData>, Box<dyn std::error::Error>> {
        // Create HMAC
        let timestamp = Local::now().timestamp_subsec_micros();
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
        println!("{}", value.to_string());
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

#[cfg(test)]
mod tests {
    use crate::FtxClient;

    #[test]
    fn test_get_markets() {
        let markets_json = r#"
            {
                "result":[
                {
                    "ask":4.0362,
                    "baseCurrency":null,
                    "bid":4.0334,
                    "change1h":0.015500754906894816,
                    "change24h":-0.02625229224978284,
                    "changeBod":-0.04503182753969569,
                    "enabled":true,
                    "highLeverageFeeExempt":false,
                    "last":4.0356,
                    "minProvideSize":1.0,
                    "name":"1INCH-PERP",
                    "postOnly":false,
                    "price":4.0356,
                    "priceIncrement":0.0001,
                    "quoteCurrency":null,
                    "quoteVolume24h":24249905.4525,
                    "restricted":false,
                    "sizeIncrement":1.0,
                    "type":"future",
                    "underlying":"1INCH",
                    "volumeUsd24h":24249905.4525
                }],
                "success": true
            }"#;
        
    }
}
