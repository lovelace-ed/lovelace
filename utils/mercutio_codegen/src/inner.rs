use itertools::Itertools;
use proc_macro2::TokenStream;
use syn::DeriveInput;

const ELEMENTS: &str = "elements";

pub fn css_inner(input: DeriveInput) -> TokenStream {
    let (mut errors, tokens): (Vec<syn::Error>, Vec<String>) =
    input
        .attrs
        .clone()
        .into_iter()
        .filter_map(|attribute| {
            attribute.parse_meta().map(|meta| {
                match meta {
                    syn::Meta::NameValue(nv) => Some(match nv.lit {
                        syn::Lit::Str(str) => Ok((nv.path, str.value())),
                        syn::Lit::Int(int) => Ok((nv.path, int.to_string())),
                        syn::Lit::Float(float) => Ok((nv.path, float.to_string())),
                        _ => Err(syn::Error::new_spanned(
                            nv,
                            "Expected a str, integer or float literal here (e.g. `#[font_family=\"sans-serif\"]` \
                            would be valid because `\"sans-serif\"` is a str, however, `#[font_family=b\"sans-serif\"]` \
                            would not beacause `b\"sans-serif\"` is a byte literal.)",
                        )),
                    }),
                    _ => None,
                }
            })
            .unwrap_or(None)

        })
        .filter_map(|item| {
            let item = item.map(|(path,  value) | {
                let key = path.segments.last().unwrap().ident.to_string();
                let css = format!("{}: {};", css_name(&key), value);
                if key == ELEMENTS {
                    return None
                }
                Some(css)
            });

            match item {
                Ok(x) => {
                    x.map(Ok)
                },
                Err(e) => {
                    Some(Err(e))
                }
            }
        })
        .partition_map(From::from);

    let tokens = tokens.into_iter().join("");

    if !errors.is_empty() {
        let mut first_error = errors.remove(0);
        for error in errors {
            first_error.combine(error)
        }
        first_error.to_compile_error();
    }

    let meta_list = if let Some(l) = input.attrs.clone().into_iter().find_map(|attr| {
        attr.parse_meta()
            .map(|meta| match meta {
                syn::Meta::List(list) => {
                    if list.path.is_ident(ELEMENTS) {
                        Some(list)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .unwrap_or(None)
    }) {
        l
    } else {
        return syn::Error::new_spanned(
            input.ident,
            "This item should have an attribute called `elements` (in the form \
                `#[elements(H1, H2, P)])`, but that attribute does not exist on this element.ƒ",
        )
        .to_compile_error();
    };

    let res = meta_list
        .nested
        .iter()
        .filter_map(|nested| match nested {
            syn::NestedMeta::Meta(meta) => match meta {
                syn::Meta::Path(path) => Some(path),
                _ => None,
            },
            syn::NestedMeta::Lit(_) => None,
        })
        .map(|segment| {
            let name = &input.ident;
            quote::quote! {
                impl ::mercutio::Apply<#name> for ::malvolio::prelude::#segment {
                    fn apply(self, _: #name) -> malvolio::prelude::#segment {
                        let string: std::borrow::Cow<'static, str> =
                            if let Some(x) = self.read_attribute("style") {
                                format!("{} {}", x, #tokens).into()
                            } else {
                                #tokens.into()
                            };
                        self.attribute(::malvolio::prelude::Style::new(string))
                    }
                }
            }
        })
        .fold(quote::quote! {}, |a, b| quote::quote! {#a #b});
    res
}

fn css_name(item: &str) -> String {
    item.replace('_', "-")
}
