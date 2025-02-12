use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    token::Comma,
    Ident, LitInt, Result,
};

struct AllTuples {
    macro_ident: Ident,
    start: usize,
    end: usize,
    idents: Vec<Ident>,
}

impl Parse for AllTuples {
    /// Parse patterns like `macro_ident[start, end]: idents...`
    fn parse(input: ParseStream) -> Result<Self> {
        let macro_ident = input.parse::<Ident>()?;

        let numbers;
        syn::bracketed!(numbers in input);
        let indices = numbers.parse_terminated(LitInt::parse, Comma)?;
        let indices: Vec<usize> = indices
            .iter()
            .map(|lit| lit.base10_parse().unwrap())
            .collect();

        input.parse::<syn::Token![:]>()?;
        let mut idents = vec![input.parse::<Ident>()?];
        while input.parse::<Comma>().is_ok() {
            idents.push(input.parse::<Ident>()?);
        }

        Ok(AllTuples {
            macro_ident,
            start: indices[0],
            end: if indices.len() >= 2 {
                indices[1]
            } else {
                indices[0]
            },
            idents,
        })
    }
}

// Copied from [bevy::all_tuples!]
// (https://github.com/bevyengine/bevy/blob/fed93a0edce9d66586dc70c1207a2092694b9a7d/crates/bevy_ecs/macros/src/lib.rs#L48-L81)
#[proc_macro]
pub fn all_tuples(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AllTuples);
    let mut ident_tuples = Vec::with_capacity(input.end - input.start);
    for i in input.start..=input.end {
        let idents = input
            .idents
            .iter()
            .map(|ident| format_ident!("{}{}", ident, i));
        if input.idents.len() < 2 {
            ident_tuples.push(quote! {
                #(#idents)*
            });
        } else {
            ident_tuples.push(quote! {
                (#(#idents),*)
            });
        }
    }

    let macro_ident = &input.macro_ident;
    let invocations = (input.start..=input.end).map(|i| {
        let ident_tuples = &ident_tuples[0..i - input.start];
        quote! {
            #macro_ident!(#(#ident_tuples),*);
        }
    });
    TokenStream::from(quote! {
        #(
            #invocations
        )*
    })
}
