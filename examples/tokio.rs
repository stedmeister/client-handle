use client_handle::async_tokio_handle;

#[async_tokio_handle]
trait MyTrait {
    fn double(input: u64) -> u64 {
        input * 2
    }

    fn double_echo(&self, input: u64) -> u64 {
        Self::double(input)
    }
}

struct Receiver;

impl MyTrait for Receiver {}

#[tokio::main]
async fn main() {
    let receiver = Receiver;
    let handle = receiver.to_async_handle();
    let result = handle.double_echo(4).await;
    assert_eq!(result, 8);
}