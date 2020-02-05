use airsim::server;
use async_std::task;

fn main() -> std::io::Result<()> {
    let res = task::block_on(server::listen("127.0.0.1:41451"));

    println!("{:#?}", res);

    Ok(())
}
