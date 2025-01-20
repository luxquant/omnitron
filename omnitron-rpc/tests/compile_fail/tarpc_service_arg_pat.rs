#[omnitron_rpc::service]
trait World {
    async fn pat((a, b): (u8, u32));
}

fn main() {}
