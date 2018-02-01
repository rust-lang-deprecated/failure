extern crate proc_macro;
extern crate syn;

#[macro_use] extern crate synstructure;
#[macro_use] extern crate quote;

use std::io::{self, Write};

decl_derive!([Fail, attributes(fail, cause)] => fail_derive);

fn fail_derive(s: synstructure::Structure) -> quote::Tokens {
    let cause_body = s.each_variant(|v| {
        if let Some(cause) = v.bindings().iter().find(is_cause) {
            quote!(return Some(#cause))
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

    #[cfg(feature = "std")]
    let fail = s.bound_impl("::failure::Fail", quote! {
        #[allow(unreachable_code)]
        fn cause(&self) -> ::std::option::Option<&::failure::Fail> {
            match *self { #cause_body }
            None
        }

        #[allow(unreachable_code)]
        fn backtrace(&self) -> ::std::option::Option<&::failure::Backtrace> {
            match *self { #bt_body }
            None
        }
    });

    #[cfg(not(feature = "std"))]
    let fail = s.bound_impl("::failure::Fail", quote! {
        #[allow(unreachable_code)]
        fn cause(&self) -> ::core::option::Option<&::failure::Fail> {
            match *self { #cause_body }
            None
        }

        #[allow(unreachable_code)]
        fn backtrace(&self) -> ::core::option::Option<&::failure::Backtrace> {
            match *self { #bt_body }
            None
        }
    });

    quote! {
        #fail
    }
}

fn is_backtrace(bi: &&synstructure::BindingInfo) -> bool {
        match bi.ast().ty {
            syn::Ty::Path(None, syn::Path { segments: ref path, .. }) => {
                path.last().map_or(false, |s| s.ident == "Backtrace" && s.parameters.is_empty())
            }
            _ => false
        }
}

fn is_cause(bi: &&synstructure::BindingInfo) -> bool {
    let mut found_cause = false;
    for attr in &bi.ast().attrs {
        if attr.name() == "cause" {
            if found_cause { panic!("Cannot have two `cause` attributes"); }
            writeln!(
                io::stderr(),
                "WARNING: failure's `#[cause]` attribute is deprecated. Use `#[fail(cause)]` instead."
            ).unwrap();
            found_cause = true;
        }
        if attr.name() == "fail" {
            if let syn::MetaItem::List(_, ref list) = attr.value {
                if let Some(&syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref word))) = list.get(0) {
                    if word == "cause" {
                        if found_cause { panic!("Cannot have two `cause` attributes"); }
                        found_cause = true;
                    }
                }
            }
        }
    }
    found_cause
}
