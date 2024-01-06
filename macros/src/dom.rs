use proc_macro2::{Delimiter, TokenStream, TokenTree};
use quote::{quote, ToTokens};

pub struct Document {
    lang: TokenTree,
    views: TokenStream,
}

impl ToTokens for Document {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Document { lang, views } = self;
        tokens.extend(quote! {
            ::ibex::view! { #views }.document(#lang)
        })
    }
}

pub fn parse_document(input: TokenStream) -> Document {
    let mut tokens = input.into_iter();

    let lang = match tokens.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => {
            let mut tokens = group.stream().into_iter();

            match tokens.next() {
                Some(TokenTree::Ident(ident)) if ident.to_string() == "lang" => (),
                token => panic!("Expected attribute name `lang`, found {:#?}. Note: Currently, only `lang` attribute is supported for <html> tag", token),
            }
            match tokens.next() {
                Some(TokenTree::Punct(punct)) if punct.to_string() == "=" => (),
                token => panic!("Expected `=`, found {:#?}", token),
            }

            let value = tokens
                .next()
                .expect("Expected attribute value expression for `lang`");

            if tokens.next().is_some() {
                panic!("Expeceted end of attribute group");
            }

            value
        }
        _ => panic!("Expected attribute group"),
    };

    let views = tokens.collect();
    Document { lang, views }
}
