use client_handle::async_tokio_handle;

#[async_tokio_handle]
trait FullTrait {
    fn simple(&self);
}

fn main() {}