use proc_macro2::{TokenStream};
use proc_macro_error::{abort};
use quote::{quote, format_ident, ToTokens};
use syn::{self, FnArg, TraitItemMethod, Ident, ReturnType, Pat, parse2};
use convert_case::{Case, Casing};

pub fn client_handle_core(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = parse2(item).unwrap();

    if let syn::Item::Trait(trayt) = &ast {
        handle_trait(&Ast { trayt })
    } else {
        abort!(ast, "The `async_tokio_handle` macro only works on traits");
    }
}

// Top Level struct that holds the AST for the trait
// that we are trying to create an async handle for
struct Ast<'a> {
    trayt: &'a syn::ItemTrait,
}

// A wrapper struct for a mthod signature
// Provides helpers to get the parameters in various
// formats
struct Method <'a> {
    sig: &'a syn::Signature,
}

impl<'a> Ast<'a> {
    // Get the name of the trait that is being wrapped
    fn trait_name(&self) -> &Ident {
        &self.trayt.ident
    }

    // Generate the name of the enum that will be used to send and
    // receive messages
    fn enum_name(&self) -> Ident {
        format_ident!("Async{}Message", self.trait_name())
    }

    // Generate the name of the struct that will be used to represent
    // the async handle
    fn handle_name(&self) -> Ident {
        format_ident!("Async{}Handle", self.trait_name())
    }

    // Gets the original trait that is being wrapped
    // So that it can be output unmodified
    fn original_trait(&self) -> TokenStream {
        self.trayt.to_token_stream().into()
    }

    // Generate a vec of all the methods in the struct
    // Returning a wrapped handle
    fn methods(&self) -> Vec<Method> {
        self.get_trait_methods()
            .iter()
            .map(|m| Method { sig: &m.sig })
            .collect()
    }

    // Get all of the trait methods that must be wrapped
    fn get_trait_methods(&'a self) -> Vec<&'a TraitItemMethod> {
        let mut methods = Vec::new();
        for item in &self.trayt.items {
            match item {
                syn::TraitItem::Method(method) => {
                    if let Some(FnArg::Receiver(_)) = method.sig.inputs.first() {
                        methods.push(method);
                    }
                },
                _ => { panic!("Can only handle trait methods") }
            }
        }
        methods
    }
}


impl<'a> Method<'a> {
    fn name(&self) -> &Ident {
        return &self.sig.ident
    }

    fn name_pascal_case(&self) -> Ident {
        format_ident!("{}", self.sig.ident.to_string().to_case(Case::Pascal))
    }

    fn typed_parameter_names_only(&self) -> Vec<&Pat> {
        let mut result = Vec::new();
        for input in &self.sig.inputs {
            match input {
                FnArg::Receiver(_) => {},
                FnArg::Typed(typed) => {
                    result.push(&*typed.pat)
                },
            }
        }
        result
    }

    fn typed_parameters(&self) -> Vec<&FnArg> {
        self.sig.inputs
            .iter()
            .filter(|arg| {
                match arg {
                    FnArg::Receiver(_) => false,
                    FnArg::Typed(_) => true,
                }
            })
            .collect()
    }

    fn return_value_type(&self) -> proc_macro2::TokenStream {
        match &self.sig.output {
            ReturnType::Default => quote!{ () },
            ReturnType::Type(_, tipe) => quote! { #tipe },
        }
    }
}


fn handle_trait(ast: &Ast) -> TokenStream {
    let message_enum = generate_message_enum(ast);

    let output = vec![
        ast.original_trait(),
        generate_struct(&ast),
        message_enum,
    ];

    let mut gen: TokenStream = TokenStream::new();
    gen.extend(output.into_iter());

    gen
}

// Generates the async handle struct and supporting code.
// 
// A trait like:
//
//  ```
//      trait MyTrait {
//         fn do_something(param: u64) -> u64;
//      }
//  ```
//
// will be turned into:
//
//  ```
//      // The struct for holding the handle
//      #[derive(Debug)]
//      struct AsyncMyTraitHandle {
//          handle: tokio::sync::mpsc::Sender<AsyncMyTraitMessage>,
//      }
//
//
//       trait ToHandle {
//           fn to_handle(self) -> AsyncMyTraitHandle;
//       }
//      
//       impl<T> ToHandle for T
//       where
//           T: MyTrait + Sync + Send + 'static
//       {
//           fn to_handle(self: T) -> AsyncMyTraitHandle {
//               AsyncMyTraitHandle::spawn(self)
//           }
//       }
//      
//       impl AsyncMyTraitHandle {
//           pub fn new(handle: tokio::sync::mpsc::Sender<AsyncMyTraitMessage>) -> Self {
//               Self { handle }
//           }
//      
//           pub fn spawn<T>(mut sync: T) -> Self
//           where
//               T: MyTrait + Sync + Send + 'static
//           {
//               let (tx, mut rx) = tokio::sync::mpsc::channel(1024);
//               tokio::spawn(async move {
//                   while let Some(msg) = rx.recv().await {
//                       match msg {
//                           AsyncMyTraitMessage::DoSomething { return_value, param } => {
//                               let result = sync.do_something(param);
//                               return_value.send(result).expect("Error calling function");
//                           }
//                       }
//                   }
//               });
//               Self { handle: tx }
//           }
//      
//           async fn do_something(&self, param: u64) -> u64 {
//               let (return_value, response) = tokio::sync::oneshot::channel();
//               self.handle.send(return_value, param).await.expect("Error when sending message to the sync code");
//               response.await.expect("Error receiving the response")
//           }
//       }
//  ```
fn generate_struct(ast: &Ast) -> TokenStream {
    let trait_name = &ast.trait_name();
    let struct_name = &ast.handle_name();
    let message_enum_name = &ast.enum_name();

    let mut async_result = Vec::new();
    let mut sync_result = Vec::new();
    for method in ast.methods() {
        let msg_name = method.name_pascal_case();
        let parameters = method.typed_parameters();
        let parameter_names = method.typed_parameter_names_only();
        let method_name = method.name();
        let return_type = method.return_value_type();

        let create_enum_call = quote! {
            #message_enum_name::#msg_name { return_value, #(#parameter_names),* }
        };

        async_result.push(quote! {
            async fn #method_name (&self, #(#parameters),*) -> #return_type {
                let (return_value, response) = tokio::sync::oneshot::channel();
                self.handle.send(#create_enum_call).await.expect("Error when sending message to the sync code");
                response.await.expect("Error receiving the response")
            }
        });

        sync_result.push(quote! {
            #message_enum_name::#msg_name { return_value, #(#parameter_names),* } => {
                let result = sync.#method_name(#(#parameter_names),*);
                return_value.send(result).expect("Error calling function");
            }
        });
    }

    quote! {
        #[derive(Debug)]
        struct #struct_name {
            handle: tokio::sync::mpsc::Sender<#message_enum_name>,
        }

        trait ToAsyncHandle {
            fn to_async_handle(self, depth: usize) -> #struct_name;
        }

        impl<T> ToAsyncHandle for T
        where
            T: #trait_name + Sync + Send + 'static
        {
            fn to_async_handle(self: T, depth: usize) -> #struct_name {
                #struct_name::spawn(self, depth)
            }
        }

        impl #struct_name {
            pub fn new(handle: tokio::sync::mpsc::Sender<#message_enum_name>) -> Self {
                Self { handle }
            }

            pub fn spawn<T>(mut sync: T, depth: usize) -> Self
            where
                T: #trait_name + Sync + Send + 'static
            {
                let (tx, mut rx) = tokio::sync::mpsc::channel(depth);
                tokio::spawn(async move {
                    while let Some(msg) = rx.recv().await {
                        match msg {
                            #(#sync_result)*
                        }
                    }
                });
                Self { handle: tx }
            }

            #(#async_result)*
        }
    }.into()

}

// Generates the enum that is responsible for [de]serialising the
// messages between the sync and async code
//
// given a function like `fn do_something(param: u64) -> u64` it should
// generate:
//
//  ```
//      #[derive(Debug)]        // required by tokio mpsc
//      enum AsyncHandleMessage {
//          DoSomething { return_value: oneshot::Sender<u64>, param: u64 },
//      }
//  ```
//
fn generate_message_enum(ast: &Ast) -> TokenStream {
    let enum_name = ast.enum_name();

    let mut enum_variants = Vec::new();
    for method in ast.methods() {
        let name = method.name_pascal_case();
        let parameters = method.typed_parameters();
        let return_type = method.return_value_type();

        enum_variants.push(quote! {
            #name {return_value: tokio::sync::oneshot::Sender<#return_type>, #(#parameters),* }
        });
    }

    quote!(
        #[derive(Debug)]
        enum #enum_name {
            #(#enum_variants),*
        }
    ).into()
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_tokens_eq(expected: &TokenStream, actual: &TokenStream) {
        let expected = expected.to_string();
        let actual = actual.to_string();
    
        if expected != actual {
            println!(
                "{}",
                colored_diff::PrettyDifference {
                    expected: &expected,
                    actual: &actual,
                }
            );
            println!("expected: {}", &expected);
            println!("actual  : {}", &actual);
            panic!("expected != actual");
        }
    }

    #[test]
    fn test_tokio_handle() {
        let before = quote! {
            trait MyTrait {
                fn ignored_associated_function();
                fn ignored_associated_function_args(input: u64) -> u64;
                fn simple(&self);
                fn echo(&self, input: u64) -> u64;
            }
        };
        let expected = quote! {
            trait MyTrait {
                fn ignored_associated_function();
                fn ignored_associated_function_args(input: u64) -> u64;
                fn simple(&self);
                fn echo(&self, input: u64) -> u64;
            }

            #[derive(Debug)]
            struct AsyncMyTraitHandle { handle: tokio::sync::mpsc::Sender<AsyncMyTraitMessage>, }

            trait ToAsyncHandle { fn to_async_handle (self, depth: usize) -> AsyncMyTraitHandle ; }

            impl<T> ToAsyncHandle for T
            where
                T: MyTrait + Sync + Send + 'static
            {
                fn to_async_handle(self: T, depth: usize) -> AsyncMyTraitHandle {
                    AsyncMyTraitHandle::spawn(self, depth)
                }
            }

            impl AsyncMyTraitHandle {
                pub fn new(handle: tokio::sync::mpsc::Sender<AsyncMyTraitMessage>) -> Self {
                    Self { handle }
                }

                pub fn spawn<T>(mut sync: T, depth: usize) -> Self
                where
                    T: MyTrait + Sync + Send + 'static
                {
                    let (tx, mut rx) = tokio::sync::mpsc::channel(depth);
                    tokio::spawn(async move {
                        while let Some(msg) = rx.recv().await {
                            match msg {
                                AsyncMyTraitMessage::Simple { return_value, } => {
                                    let result = sync.simple();
                                    return_value.send(result).expect("Error calling function");
                                }
                                AsyncMyTraitMessage::Echo { return_value, input } => {
                                    let result = sync.echo(input);
                                    return_value.send(result).expect("Error calling function");
                                }
                            }
                        }
                    });
                    Self { handle: tx }
                }

                async fn simple(&self, ) -> () {
                    let (return_value, response) = tokio::sync::oneshot::channel();
                    self.handle.send(AsyncMyTraitMessage::Simple{ return_value, }).await.expect("Error when sending message to the sync code");
                    response.await.expect("Error receiving the response")
                }

                async fn echo(&self, input: u64) -> u64 {
                    let (return_value, response) = tokio::sync::oneshot::channel();
                    self.handle.send(AsyncMyTraitMessage::Echo{ return_value, input }).await.expect("Error when sending message to the sync code");
                    response.await.expect("Error receiving the response")
                }
            }

            #[derive(Debug)]
            enum AsyncMyTraitMessage {
                Simple { return_value: tokio::sync::oneshot::Sender<()>, },
                Echo { return_value: tokio::sync::oneshot::Sender<u64>, input: u64 }
            }


        };
        let after = client_handle_core(quote!(), before);
        assert_tokens_eq(&expected, &after);
    }
}
