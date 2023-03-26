use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Data, DeriveInput, Fields, FieldsNamed, LitStr, Variant,
};

#[proc_macro_derive(EnumContract, attributes(topic))]
pub fn derive_enum2contract(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let data = match input.data {
        Data::Enum(data) => data,
        _ => {
            return syn::Error::new(input.span(), "enum2contract only supports enums")
                .to_compile_error()
                .into()
        }
    };

    let mut message_functions = proc_macro2::TokenStream::new();
    let mut payloads = proc_macro2::TokenStream::new();

    for variant in data.variants.iter() {
        match variant.fields {
            Fields::Unit => {
                let topic = match parse_topic_attribute(variant) {
                    Ok(value) => value,
                    Err(error) => return error.to_compile_error().into(),
                };

                // Generate the payload struct for the variant.
                let payload_name =
                    Ident::new(&format!("{}Payload", variant.ident), variant.ident.span());
                let payload_struct = quote! {
                    #[derive(Default, Debug, PartialEq)]
                    pub struct #payload_name;
                };
                payloads.extend(payload_struct);

                let payload_type = quote! { #payload_name };
                let payload_default = quote! { #payload_name::default() };
                let create_name = Ident::new(
                    &to_snake_case(&variant.ident.to_string()),
                    variant.ident.span(),
                );
                let topic_string = &topic.value();
                let args = extract_substrings(topic_string);
                let topic_string = remove_substrings(&topic.value(), &args);
                let args: Vec<_> = args
                    .iter()
                    .map(|arg| Ident::new(arg, Span::call_site()))
                    .collect();

                let message_function = quote! {
                    pub fn #create_name(#(#args: &str),*) -> (String, #payload_type) {
                        (format!(#topic_string, #(#args),*), #payload_default)
                    }
                };
                message_functions.extend(message_function);
            }

            Fields::Named(FieldsNamed { ref named, .. }) => {
                let topic = match parse_topic_attribute(variant) {
                    Ok(value) => value,
                    Err(error) => return error.to_compile_error().into(),
                };

                // Generate the payload struct for the variant.
                let payload_name =
                    Ident::new(&format!("{}Payload", variant.ident), variant.ident.span());
                let payload_struct = quote! {
                    #[derive(Default, Debug, PartialEq)]
                    pub struct #payload_name {
                        #named
                    }
                };
                payloads.extend(payload_struct);

                let payload_type = quote! { #payload_name };
                let payload_default = quote! { #payload_name::default() };
                let create_name = Ident::new(
                    &to_snake_case(&variant.ident.to_string()),
                    variant.ident.span(),
                );
                let topic_string = &topic.value();
                let args = extract_substrings(topic_string);
                let topic_string = remove_substrings(&topic.value(), &args);
                let args: Vec<_> = args
                    .iter()
                    .map(|arg| Ident::new(arg, Span::call_site()))
                    .collect();

                let message_function = quote! {
                    pub fn #create_name(#(#args: &str),*) -> (String, #payload_type) {
                        (format!(#topic_string, #(#args),*), #payload_default)
                    }
                };
                message_functions.extend(message_function);
            }

            _ => {
                return syn::Error::new(
                    variant.span(),
                    "enum2contract is only implemented for named-field enums",
                )
                .to_compile_error()
                .into()
            }
        };
    }

    let expanded = quote! {
        #payloads

        impl #name {
            #message_functions
        }
    };

    TokenStream::from(expanded)
}

fn parse_topic_attribute(variant: &Variant) -> Result<LitStr, syn::Error> {
    let mut topic = None;
    for attr in &variant.attrs {
        if attr.path.is_ident("topic") {
            match attr.parse_args::<LitStr>() {
                Ok(literal) => topic = Some(literal),
                Err(_) => {
                    return Err(syn::Error::new(
                        attr.path.span(),
                        r#"The 'topic' attribute is missing a String argument. Example: #[topic("system/{id}/start")] "#,
                    ));
                }
            }
        }
    }
    topic.ok_or_else(|| {
        syn::Error::new(
            variant.span(),
            r#"The 'topic' attribute is required. Example: #[topic("system/{id}/start")]"#,
        )
    })
}

fn extract_substrings(s: &str) -> Vec<&str> {
    s.split('{')
        .skip(1)
        .filter_map(|substr| substr.split_once('}'))
        .map(|(outer, _)| outer)
        .collect()
}

fn remove_substrings(s: &str, substrings: &[&str]) -> String {
    let mut result = String::from(s);
    for substring in substrings {
        result = result.replace(&format!("{{{}}}", substring), "{}");
    }
    result
}

fn to_snake_case(input: &str) -> String {
    input
        .chars()
        .enumerate()
        .flat_map(|(i, c)| {
            if c.is_uppercase() {
                let mut s = String::new();
                if i != 0 && !input.is_empty() && input.chars().next().unwrap().is_uppercase() {
                    s.push('_');
                }
                s.push_str(&c.to_lowercase().to_string());
                std::iter::once(s)
            } else {
                std::iter::once(c.to_string())
            }
        })
        .collect::<Vec<String>>()
        .join("")
}
