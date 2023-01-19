use client_handle::async_tokio_handle;

#[async_tokio_handle]
struct SomeStruct;

#[async_tokio_handle]
enum SomeEnum {}

fn main() {}