use airsim::client::Client;
use airsim::message::Request;
use async_std::task;

async fn run_client() -> std::io::Result<()> {
    let address = "127.0.0.1:41451";
    let mut client = Client::connect(address).await?;

    let response = client
        .request(Request {
            id: 1,
            method: "dostuff".to_string(),
            params: Vec::new(),
        })
        .await?;
    println!("{:#?}", response);
    Ok(())
}
fn main() -> std::io::Result<()> {
    task::block_on(run_client())
}
