extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn new(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let f = syn::parse_macro_input!(item as syn::ItemFn);

    let attrs = &f.attrs;
    let ident = &f.ident;
    let output = &f.decl.output;
    let block = &f.block;
    (quote::quote! {
        #(#attrs)*
        #[test]
        fn #ident() #output {
            drop(env_logger::Builder::from_default_env()
                 .default_format_timestamp(false)
                 .try_init());
            #block
        }
    })
    .into()
}
