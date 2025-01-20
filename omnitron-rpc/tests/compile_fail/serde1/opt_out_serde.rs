#![allow(deprecated)]

use std::fmt::Formatter;

#[omnitron_rpc::service(derive_serde = false)]
trait Foo {
    async fn foo();
}

fn foo(f: &mut Formatter) {
    let x = FooRequest::Foo {};
    omnitron_rpc::serde::Serialize::serialize(&x, f);
}

fn main() {}
