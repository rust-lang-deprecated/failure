extern crate proc_macro;
extern crate syn;

#[macro_use] extern crate synstructure;
#[macro_use] extern crate quote;

decl_derive!([Fail, attributes(error_msg, cause)] => fail_derive);

fn fail_derive(s: synstructure::Structure) -> quote::Tokens {
    let fail_body = s.each_variant(|v| {
        let msg = find_error_msg(&v.ast().attrs);
        if msg.is_empty() {
            panic!("Expected at least one argument to error_msg");
        }

        let s = &msg[0];
        let args = msg[1..].iter().map(|arg| match *arg {
            syn::NestedMetaItem::Literal(syn::Lit::Int(i, _)) => {
                let bi = &v.bindings()[i as usize];
                quote!(#bi)
            }
            syn::NestedMetaItem::MetaItem(syn::MetaItem::Word(ref id)) => {
                for bi in v.bindings() {
                    if bi.ast().ident.as_ref() == Some(id) {
                        return quote!(#bi);
                    }
                }
                panic!("Couldn't find a field with this name!");
            }
            _ => panic!("Invalid argument to error_msg attribute!"),
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

    let conversions = s.variants().iter().filter_map(|v| {
        if let Some(cause) = v.bindings().iter().find(is_cause) {
            let cause_field = cause.ast().ident.clone().unwrap_or(syn::Ident::new("__cause"));
            let cause_ty = &cause.ast().ty;
            let defaults = v.bindings().iter()
                            .enumerate()
                            .filter(|&(_, bi)| bi != cause)
                            .map(|(idx, bi)| bi.ast().ident.clone().unwrap_or(syn::Ident::new(format!("__{}", idx))));
            let constructor = {
                let mut v = v.clone();
                v.bind_with(|_| synstructure::BindStyle::Move);
                v.binding_name(|field, idx| {
                    if field == cause.ast() {
                        cause_field.clone()
                    } else {
                        field.ident.clone().unwrap_or(syn::Ident::new(format!("__{}", idx)))
                    }
                });
                v.pat()
            };
            Some(s.bound_impl(quote!(::std::convert::From<#cause_ty>), quote! {
                fn from(#cause_field: #cause_ty) -> Self {
                    #(let #defaults = ::std::default::Default::default();)*
                    #constructor
                }
            }))
        } else { None }
    });

    let fail = s.bound_impl("::failure::Fail", quote! {
        #[allow(unreachable_code)]
        fn fail(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self { #fail_body }
            write!(f, "An error has occurred.")
        }

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
        #(#conversions)*
    }
}

fn find_error_msg(attrs: &[syn::Attribute]) -> &[syn::NestedMetaItem] {
    let mut error_msg = None;
    for attr in attrs {
        if attr.name() == "error_msg" {
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
