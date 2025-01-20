#![deny(warnings)]

#[omnitron_rpc::service(derive_serde = true)]
trait Foo {
    async fn foo();
}

fn main() {}
