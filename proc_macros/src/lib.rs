use proc_macro::{Literal, TokenStream, TokenTree};

#[proc_macro]
pub fn lower_ident_str(stream: TokenStream) -> TokenStream {
    stream
    .into_iter()
    .map(|token| match token {
        TokenTree::Ident(ident) => {
            TokenTree::Literal(Literal::string(&ident.to_string().to_lowercase()))
        }
        TokenTree::Literal(literal) => {
            TokenTree::Literal(Literal::string(&literal.to_string().to_lowercase()))
        }
        _ => token,
    })
    .collect()

}