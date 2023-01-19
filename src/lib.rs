#![doc = include_str!("../README.md")]

/// Generates a client handle that uses tokio channels
/// to send the messages between tasks.
/// 
/// The macro should be applied to a trait as shown below:
/// 
/// ```rust
/// use client_handle::async_tokio_handle;
/// 
/// #[async_tokio_handle]
/// trait MyTrait {
///     fn double_echo(&self, input: u64) -> u64 {
///         input * 2
///     }
/// }
///
/// struct Receiver;
///
/// impl MyTrait for Receiver {}
///
/// #[tokio::main]
/// async fn main() {
///     let receiver = Receiver;
///     let handle = receiver.to_async_handle(32);
///     let result = handle.double_echo(4).await;
///     assert_eq!(result, 8);
/// }
/// ```
pub use client_handle_derive::async_tokio_handle;