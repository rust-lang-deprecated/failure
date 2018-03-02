extern crate proc_macro;

#[macro_use] extern crate syn;
#[macro_use] extern crate synstructure;
#[macro_use] extern crate quote;

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
    let fail = s.bound_impl(quote!(::failure::Fail), quote! {
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
    let fail = s.bound_impl(quote!(::failure::Fail), quote! {
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
            syn::Type::Path(syn::TypePath { qself: None, path: syn::Path { ref segments, .. } }) => {
                segments.last().map_or(false, |p| p.value().ident == "Backtrace" && p.value().arguments.is_empty())
            }
            _ => false
        }
}

fn is_cause(bi: &&synstructure::BindingInfo) -> bool {
    let mut found_cause = false;
    for attr in &bi.ast().attrs {
        if attr.path == parse_quote!(fail) {
            if let Some(syn::Meta::List(ref list)) = attr.interpret_meta() {
                if let Some(pair) = list.nested.first() {
                    if let &syn::NestedMeta::Meta(syn::Meta::Word(ref word)) = pair.into_value() {
                        if word == "cause" {
                            if found_cause { panic!("Cannot have two `cause` attributes"); }
                            found_cause = true;
                        }
                    }
                }
            }
        }
    }
    found_cause
}
