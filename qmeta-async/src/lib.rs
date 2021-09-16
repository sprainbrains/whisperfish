extern crate proc_macro;
use quote::quote;

#[proc_macro_attribute]
pub fn with_executor(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    // let args = syn::parse_macro_input!(args as syn::AttributeArgs);

    let attrs = &input.attrs;
    let sig = &input.sig;
    let vis = &input.vis;
    // let name = &input.sig.ident;
    // let ret = &input.sig.ret;
    let body = &input.block;

    let result = quote! {
        #(#attrs)*
        #vis #sig {
            crate::gui::with_executor(move || {
                #body
            })
        }
    };
    result.into()
}
