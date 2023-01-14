#![doc = include_str!("../README.md")]

use client_handle_core::client_handle_core;
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn async_tokio_handle(attr: TokenStream, item: TokenStream) -> TokenStream {
    client_handle_core(attr.into(), item.into()).into()
}