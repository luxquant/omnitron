use omnitron_rpc::client;

#[omnitron_rpc::service]
trait World {
    async fn hello(name: String) -> String;
}

fn main() {
    let (client_transport, _) = omnitron_rpc::transport::channel::unbounded();

    #[deny(unused_must_use)]
    {
        WorldClient::new(client::Config::default(), client_transport).dispatch;
    }
}
