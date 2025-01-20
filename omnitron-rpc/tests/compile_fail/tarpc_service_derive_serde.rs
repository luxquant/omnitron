#[omnitron_rpc::service(derive_serde = loop {})]
trait World {
    async fn hello();
}

fn main() {}
