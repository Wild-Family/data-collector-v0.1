extern crate dotenv;
#[macro_use]
extern crate dotenv_codegen;

use ftx_client::FtxClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get ENV
    let ftx_api_key = dotenv!("FTX_API_KEY").to_string();
    let ftx_api_secret = dotenv!("FTX_API_SECRET").to_string();

    let ftx_client = FtxClient::new(ftx_api_key, ftx_api_secret);

    let market_datas = ftx_client.get_markets().await?;

    for (k, _v) in &market_datas {
        println!("{}", k);
    }

    println!("{:?}", market_datas["SOL-PERP"]);

    Ok(())
}