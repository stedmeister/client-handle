use client_handle::async_tokio_handle;

#[derive(Debug, Clone, PartialEq)]
struct MyType(u64);

#[async_tokio_handle]
trait DummyTrait {
    // fn ignored_associated_function();
    fn increment(&mut self, a: u64) -> MyType;
    fn output(&self);
}

struct SyncCode { item: u64 }

impl DummyTrait for SyncCode {
    fn increment(&mut self, a: u64) -> MyType {
        self.item += a;
        MyType(self.item)
    }

    fn output(&self) {
        println!("!!!!Here we are in the sync code {}", self.item);
    }
}


#[tokio::main]
async fn main() {
    let sync = SyncCode{ item: 5 };
    let handle = sync.to_async_handle();
    handle.output().await;
    handle.increment(4).await;
    handle.output().await;
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_increment() {

    }
}
