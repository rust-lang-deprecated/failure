extern crate proc_macro;
extern crate syn;

#[macro_use] extern crate synstructure;
#[macro_use] extern crate quote;

decl_derive!([Fail, attributes(fail, cause)] => fail_derive);

fn fail_derive(s: synstructure::Structure) -> quote::Tokens {
    let display_body = s.each_variant(|v| {
        let msg = find_error_msg(&v.ast().attrs);
        if msg.is_empty() {
            panic!("Expected at least one argument to fail attribute");
        }

        let s = match msg[0] {
            syn::NestedMetaItem::MetaItem(syn::MetaItem::NameValue(ref i, ref lit)) if i == "display" => {
                lit.clone()
            }
            _ => panic!("Fail attribute must begin `display = \"\"` to control the Display message."),
        };
        let args = msg[1..].iter().map(|arg| match *arg {
            syn::NestedMetaItem::Literal(syn::Lit::Int(i, _)) => {
                let bi = &v.bindings()[i as usize];
                quote!(#bi)
            }
            syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref id)) => {
                if id.as_ref().starts_with("_") {
                    if let Ok(idx) = id.as_ref()[1..].parse::<usize>() {
                        let bi = &v.bindings()[idx];
                        return quote!(#bi)
                    }
                }
                for bi in v.bindings() {
                    if bi.ast().ident.as_ref() == Some(id) {
                        return quote!(#bi);
                    }
                }
                panic!("Couldn't find a field with this name!");
            }
            _ => panic!("Invalid argument to fail attribute!"),
        });

        quote! {
            return write!(f, #s #(, #args)*)
        }
    });

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

    let display = s.bound_impl("::std::fmt::Display", quote! {
        #[allow(unreachable_code)]
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self { #display_body }
            write!(f, "An error has occurred.")
        }
    });

    let fail = s.bound_impl("::failure::Fail", quote! {
        #[allow(unreachable_code)]
        fn cause(&self) -> Option<&::failure::Fail> {
            match *self { #cause_body }
            None
        }

        #[allow(unreachable_code)]
        fn backtrace(&self) -> ::std::option::Option<&::failure::Backtrace> {
            match *self { #bt_body }
            None
        }
    });

    quote! {
        #fail
        #display
    }
}

fn find_error_msg(attrs: &[syn::Attribute]) -> &[syn::NestedMetaItem] {
    let mut error_msg = None;
    for attr in attrs {
        if attr.name() == "fail" {
            if error_msg.is_some() {
                panic!("Cannot have two error_msg attributes")
            } else {
                if let syn::MetaItem::List(_, ref list)  = attr.value {
                    error_msg = Some(&list[..]);
                } else {
                    panic!("error_msg must take a list in parantheses")
                }
            }
        }
    }
    error_msg.expect("Must have attribute error_msg")
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
    bi.ast().attrs.iter().any(|attr| attr.name() == "cause")
}
