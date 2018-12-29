extern crate proc_macro2;
extern crate syn;

#[macro_use]
extern crate synstructure;
#[macro_use]
extern crate quote;

use proc_macro2::{TokenStream, Span};
use syn::LitStr;
use syn::spanned::Spanned;

#[derive(Debug)]
struct Error(TokenStream);

impl Error {
    fn new(span: Span, message: &str) -> Error {
        Error(quote_spanned! { span =>
            compile_error!(#message);
        })
    }

    fn into_tokens(self) -> TokenStream {
        self.0
    }
}

decl_derive!([Fail, attributes(fail, cause)] => fail_derive);

fn fail_derive(s: synstructure::Structure) -> TokenStream {
    match fail_derive_impl(s) {
        Err(err) => err.into_tokens(),
        Ok(tokens) => tokens,
    }
}

fn fail_derive_impl(s: synstructure::Structure) -> Result<TokenStream, Error> {
    let make_dyn = if cfg!(has_dyn_trait) {
        quote! { &dyn }
    } else {
        quote! { & }
    };

    let ty_name = LitStr::new(&s.ast().ident.to_string(), Span::call_site());

    let cause_body = s.each_variant(|v| {
        if let Some(cause) = v.bindings().iter().find(is_cause) {
            quote!(return Some(::failure::AsFail::as_fail(#cause)))
        } else {
            quote!(return None)
        }
    });

    let bt_body = s.each_variant(|v| {
        if let Some(bi) = v.bindings().iter().find(is_backtrace) {
            quote!(return Some(#bi))
        } else {
            quote!(return None)
        }
    });

    let fail = s.unbound_impl(
        quote!(::failure::Fail),
        quote! {
            fn name(&self) -> Option<&str> {
                Some(concat!(module_path!(), "::", #ty_name))
            }

            #[allow(unreachable_code)]
            fn cause(&self) -> ::failure::_core::option::Option<#make_dyn(::failure::Fail)> {
                match *self { #cause_body }
                None
            }

            #[allow(unreachable_code)]
            fn backtrace(&self) -> ::failure::_core::option::Option<&::failure::Backtrace> {
                match *self { #bt_body }
                None
            }
        },
    );
    let display = display_body(&s)?.map(|display_body| {
        s.unbound_impl(
            quote!(::failure::_core::fmt::Display),
            quote! {
                #[allow(unreachable_code)]
                fn fmt(&self, f: &mut ::failure::_core::fmt::Formatter) -> ::failure::_core::fmt::Result {
                    match *self { #display_body }
                    write!(f, "An error has occurred.")
                }
            },
        )
    });

    Ok(quote! {
        #fail
        #display
    })
}

fn display_body(s: &synstructure::Structure) -> Result<Option<quote::__rt::TokenStream>, Error> {
    let mut msgs = s.variants().iter().map(|v| find_error_msg(&v.ast().attrs));
    if msgs.all(|msg| msg.map(|m| m.is_none()).unwrap_or(true)) {
        return Ok(None);
    }

    let mut tokens = TokenStream::new();
    for v in s.variants() {
        let msg =
            find_error_msg(&v.ast().attrs)?
              .ok_or_else(|| Error::new(
                  v.ast().ident.span(),
                  "All variants must have display attribute."
              ))?;
        if msg.nested.is_empty() {
            return Err(Error::new(
                msg.span(),
                "Expected at least one argument to fail attribute"
            ));
        }

        let format_string = match msg.nested[0] {
            syn::NestedMeta::Meta(syn::Meta::NameValue(ref nv)) if nv.ident == "display" => {
                nv.lit.clone()
            }
            _ => {
                return Err(Error::new(
                    msg.span(),
                    "Fail attribute must begin `display = \"\"` to control the Display message."
                ));
            }
        };
        let args = msg.nested.iter().skip(1).map(|arg| match *arg {
            syn::NestedMeta::Literal(syn::Lit::Int(ref i)) => {
                let bi = &v.bindings()[i.value() as usize];
                Ok(quote!(#bi))
            }
            syn::NestedMeta::Meta(syn::Meta::Word(ref id)) => {
                let id_s = id.to_string();
                if id_s.starts_with("_") {
                    if let Ok(idx) = id_s[1..].parse::<usize>() {
                        let bi = match v.bindings().get(idx) {
                            Some(bi) => bi,
                            None => {
                                return Err(Error::new(
                                    arg.span(),
                                    &format!(
                                        "display attempted to access field `{}` in `{}::{}` which \
                                     does not exist (there are {} field{})",
                                        idx,
                                        s.ast().ident,
                                        v.ast().ident,
                                        v.bindings().len(),
                                        if v.bindings().len() != 1 { "s" } else { "" }
                                    )
                                ));
                            }
                        };
                        return Ok(quote!(#bi));
                    }
                }
                for bi in v.bindings() {
                    if bi.ast().ident.as_ref() == Some(id) {
                        return Ok(quote!(#bi));
                    }
                }
                return Err(Error::new(
                    arg.span(),
                    &format!(
                        "Couldn't find field `{}` in `{}::{}`",
                        id,
                        s.ast().ident,
                        v.ast().ident
                    )
                ));
            }
            ref arg => {
                return Err(Error::new(
                    arg.span(),
                    "Invalid argument to fail attribute!"
                ));
            },
        });
        let args = args.collect::<Result<Vec<_>, _>>()?;

        let pat = v.pat();
        tokens.extend(quote!(#pat => { return write!(f, #format_string #(, #args)*) }));
    }
    Ok(Some(tokens))
}

fn find_error_msg(attrs: &[syn::Attribute]) -> Result<Option<syn::MetaList>, Error> {
    let mut error_msg = None;
    for attr in attrs {
        if let Some(meta) = attr.interpret_meta() {
            if meta.name() == "fail" {
                if error_msg.is_some() {
                    return Err(Error::new(
                        meta.span(),
                        "Cannot have two display attributes"
                    ));
                } else {
                    if let syn::Meta::List(list) = meta {
                        error_msg = Some(list);
                    } else {
                        return Err(Error::new(
                            meta.span(),
                            "fail attribute must take a list in parentheses"
                        ));
                    }
                }
            }
        }
    }
    Ok(error_msg)
}

fn is_backtrace(bi: &&synstructure::BindingInfo) -> bool {
    match bi.ast().ty {
        syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                segments: ref path, ..
            },
        }) => path.last().map_or(false, |s| {
            s.value().ident == "Backtrace" && s.value().arguments.is_empty()
        }),
        _ => false,
    }
}

fn is_cause(bi: &&synstructure::BindingInfo) -> bool {
    let mut found_cause = false;
    for attr in &bi.ast().attrs {
        if let Some(meta) = attr.interpret_meta() {
            if meta.name() == "cause" {
                if found_cause {
                    panic!("Cannot have two `cause` attributes");
                }
                found_cause = true;
            }
            if meta.name() == "fail" {
                if let syn::Meta::List(ref list) = meta {
                    if let Some(ref pair) = list.nested.first() {
                        if let &&syn::NestedMeta::Meta(syn::Meta::Word(ref word)) = pair.value() {
                            if word == "cause" {
                                if found_cause {
                                    panic!("Cannot have two `cause` attributes");
                                }
                                found_cause = true;
                            }
                        }
                    }
                }
            }
        }
    }
    found_cause
}
