#![allow(deprecated)]
#[omnitron_rpc::service(derive = [Clone], derive_serde = true)]
trait Foo {
    async fn foo();
}

fn main() {}
