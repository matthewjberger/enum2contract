use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, Variant, parse_macro_input, spanned::Spanned};

#[proc_macro_derive(EnumContract, attributes(topic))]
pub fn derive_enum2contract(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let data = match input.data {
        Data::Enum(data) => data,
        _ => {
            return syn::Error::new(input.span(), "enum2contract only supports enums")
                .to_compile_error()
                .into();
        }
    };

    let mut message_functions = proc_macro2::TokenStream::new();
    let mut payloads = proc_macro2::TokenStream::new();

    for variant in data.variants.iter() {
        match expand_variant(variant) {
            Ok((payload, message_function)) => {
                payloads.extend(payload);
                message_functions.extend(message_function);
            }
            Err(error) => return error.to_compile_error().into(),
        }
    }

    let expanded = quote! {
        #payloads

        impl #name {
            #message_functions
        }
    };

    TokenStream::from(expanded)
}

type VariantTokens = (proc_macro2::TokenStream, proc_macro2::TokenStream);

fn expand_variant(variant: &Variant) -> Result<VariantTokens, syn::Error> {
    let topic = parse_topic_attribute(variant)?;
    let payload_name = Ident::new(&format!("{}Payload", variant.ident), variant.ident.span());

    let payload_struct = match &variant.fields {
        Fields::Unit => quote! {
            #[derive(
                Default,
                Debug,
                Clone,
                PartialEq,
                enum2contract::serde::Serialize,
                enum2contract::serde::Deserialize,
            )]
            #[serde(crate = "enum2contract::serde")]
            pub struct #payload_name;
        },
        Fields::Named(named_fields) => {
            let mut fields = proc_macro2::TokenStream::new();
            for field in named_fields.named.iter() {
                fields.extend(quote! { pub #field, });
            }
            quote! {
                #[derive(
                    Default,
                    Debug,
                    Clone,
                    PartialEq,
                    enum2contract::serde::Serialize,
                    enum2contract::serde::Deserialize,
                )]
                #[serde(crate = "enum2contract::serde")]
                pub struct #payload_name {
                    #fields
                }
            }
        }
        Fields::Unnamed(_) => {
            return Err(syn::Error::new(
                variant.span(),
                "enum2contract is only implemented for unit and named-field enum variants",
            ));
        }
    };

    let payload = quote! {
        #payload_struct

        impl #payload_name {
            pub fn to_json(&self) -> Result<String, enum2contract::serde_json::Error> {
                enum2contract::serde_json::to_string(self)
            }

            pub fn from_json(json: &str) -> Result<Self, enum2contract::serde_json::Error> {
                enum2contract::serde_json::from_str(json)
            }

            pub fn to_bytes(&self) -> Result<Vec<u8>, enum2contract::postcard::Error> {
                enum2contract::postcard::to_allocvec(self)
            }

            pub fn from_bytes(bytes: &[u8]) -> Result<Self, enum2contract::postcard::Error> {
                enum2contract::postcard::from_bytes(bytes)
            }
        }
    };

    let ident_name = to_snake_case(&variant.ident.to_string());
    let create_message = Ident::new(&ident_name, variant.ident.span());
    let create_topic = Ident::new(&format!("{ident_name}_topic"), variant.ident.span());
    let placeholders = extract_placeholders(&topic)?;
    let format_string = remove_placeholders(&topic.value(), &placeholders);
    let parameters: Vec<Ident> = placeholders
        .iter()
        .map(|placeholder| Ident::new(placeholder, Span::call_site()))
        .collect();

    let message_function = quote! {
        pub fn #create_message(#(#parameters: &str),*) -> (String, #payload_name) {
            (Self::#create_topic(#(#parameters),*), #payload_name::default())
        }

        pub fn #create_topic(#(#parameters: &str),*) -> String {
            format!(#format_string, #(#parameters),*)
        }
    };

    Ok((payload, message_function))
}

fn parse_topic_attribute(variant: &Variant) -> Result<LitStr, syn::Error> {
    let mut topic = None;
    for attr in &variant.attrs {
        if attr.path().is_ident("topic") {
            match attr.parse_args::<LitStr>() {
                Ok(literal) => topic = Some(literal),
                Err(_) => {
                    return Err(syn::Error::new(
                        attr.path().span(),
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

fn extract_placeholders(topic: &LitStr) -> Result<Vec<String>, syn::Error> {
    let value = topic.value();
    let mut placeholders = Vec::new();
    for segment in value.split('{').skip(1) {
        let Some((placeholder, _)) = segment.split_once('}') else {
            continue;
        };
        if placeholder.is_empty() {
            return Err(syn::Error::new(
                topic.span(),
                "topic placeholders must be named, like {id}",
            ));
        }
        if syn::parse_str::<Ident>(placeholder).is_err() {
            return Err(syn::Error::new(
                topic.span(),
                format!("topic placeholder '{{{placeholder}}}' is not a valid identifier"),
            ));
        }
        if placeholders.iter().any(|existing| existing == placeholder) {
            return Err(syn::Error::new(
                topic.span(),
                format!("topic placeholder '{{{placeholder}}}' appears more than once"),
            ));
        }
        placeholders.push(placeholder.to_string());
    }
    Ok(placeholders)
}

fn remove_placeholders(topic: &str, placeholders: &[String]) -> String {
    let mut result = String::from(topic);
    for placeholder in placeholders {
        result = result.replace(&format!("{{{placeholder}}}"), "{}");
    }
    result
}

fn to_snake_case(input: &str) -> String {
    let characters: Vec<char> = input.chars().collect();
    let mut result = String::new();
    for (index, character) in characters.iter().enumerate() {
        if character.is_uppercase() {
            let previous_is_lowercase = index > 0 && characters[index - 1].is_lowercase();
            let previous_is_digit = index > 0 && characters[index - 1].is_ascii_digit();
            let previous_is_uppercase = index > 0 && characters[index - 1].is_uppercase();
            let next_is_lowercase = characters
                .get(index + 1)
                .is_some_and(|next| next.is_lowercase());
            if previous_is_lowercase
                || previous_is_digit
                || (previous_is_uppercase && next_is_lowercase)
            {
                result.push('_');
            }
            result.extend(character.to_lowercase());
        } else {
            result.push(*character);
        }
    }
    result
}
