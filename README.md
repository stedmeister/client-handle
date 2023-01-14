client-handle
==========

A common pattern with writting multithreaded / asynchronous code is to allow a
thread / task to own a resource and to send messages through a channel to
access it.  e.g. From the tokio redis example: https://tokio.rs/tokio/tutorial/channels.

The pattern is along the lines of:

* Create a message enum.
* Create a channel
* Spawn a background task to read from the `rx` Receiver.
* Send messages from one or more `tx` Senders
* Use a oneshot channel sent with the message to return the reponse

To provide an ergonomic handle, I also often end up wrapper the `tx` Sender and
 duplicating all of the client functions. As shown below, this results in
a lot of boiler plate.

```rust ignore
// Generate the message enum
enum Command {
    Get {
        reponse: oneshot::Sender<String>,
        key: String,
    }
}

// Create a channel and
// Spawn a receiver task
let (tx, mut rx) = mpsc::channel(32);
tokio::spawn(async move {
    while let Some(cmd) = rx.recv().await {
    use Command::*;

    match cmd {
        Get { reponse, key } => {
            let value = get_value(&key).await;
            let _ = response.send(value);
        }
    }
}

// Send messages to the channel using an ergonic client
struct Handle {
    tx: mpsc::Sender<Command>,
}

impl Handle {
    async fn get(&self, key: &String) {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get {
            key: key.to_string(),
            resp: resp_tx,
        };

        // Send the GET request
        tx.send(cmd).await.unwrap();

        // Await the response
        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    }
}
```

The boiler plate in question is the duplication in:

* The receiving code to unpack the message and call the actual implementation
* The definition of the enum
* The impl of the client handle

It should be possible to provide only one of the above parts code and derive the
others.  This is where `client-handle` comes in as it will derive the mesage
format based on a trait that the receiving code has to adere to.

In short, the code above could be replaced with the following:

```rust ignore
use client_handle::async_tokio_handle;

#[async_tokio_handle]
trait KvCommand {
    fn get(&self, key: String) -> String {
        self.get_value(&key)
    }
}
```

And it can be used as follows:

```rust ignore
// create a struct for the trait
struct KvReceiver { /* data owned by the receiver */ };

impl KvCommand for KvReceiver {
    // Nothing to do here as the trait has default implemenations
}

#[tokio::main]
async fn main() {
    let receiver = KvReceiver;
    let handle = receiver.to_async_handle();
    let result = handle.get("foo".to_string()).await;
}
```

There are other examples in the code.  For the full details of the code
generated, please see the unit tests in the `client-handle-core` crate.

Why create a sync trait?
========================

It was chosen to place the macro on the trait for the following reasons:

* Decorating the enum would have involved having users create "magic strings"
  for return values.
* Using a trait allows for tools like `automock` to be used for testing


Acknowledgements
================

Please see the (notes)[./NOTES.md] file for details on resources used to create
this proc macro.