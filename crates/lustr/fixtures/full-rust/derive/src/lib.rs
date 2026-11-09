use proc_macro::{TokenStream, TokenTree};

/// A real host procedural macro. Its output crosses a crate boundary.
#[proc_macro_derive(Answer)]
pub fn answer(input: TokenStream) -> TokenStream {
    let mut tokens = input.into_iter();
    while let Some(token) = tokens.next() {
        if matches!(token, TokenTree::Ident(ref ident) if ident.to_string() == "struct") {
            let Some(TokenTree::Ident(name)) = tokens.next() else {
                panic!("Answer expects a named struct");
            };
            return format!("impl {name} {{ pub fn derived() -> u32 {{ 41 }} }}")
                .parse()
                .unwrap();
        }
    }
    panic!("Answer expects a struct");
}
