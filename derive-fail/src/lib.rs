extern crate proc_macro;
extern crate syn;

#[macro_use] extern crate synstructure;
#[macro_use] extern crate quote;

decl_derive!([Fail, attributes(error_msg)] => fail_derive);

fn fail_derive(s: synstructure::Structure) -> quote::Tokens {
    let fail_body = s.each_variant(|v| {
        let msg = find_error_msg(&v.ast().attrs).unwrap();
        if msg.is_empty() {
            panic!("Expected at least one argument to error_msg");
        }

        let s = &msg[0];
        let args = msg[1..].iter().map(|arg| match *arg {
            syn::NestedMetaItem::Literal(
                syn::Lit::Int(i, _)
            ) => {
                let bi = &v.bindings()[i as usize];
                quote!(#bi)
            }
            syn::NestedMetaItem::MetaItem(
                syn::MetaItem::Word(ref id)
            ) => {
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
            write!(f, #s #(, #args)*)
        }
    });

    let bt_body = s.each_variant(|v| {
        for bi in v.bindings() {
            if is_backtrace(bi.ast()) {
                return quote!(return Some(#bi););
            }
        }
        quote!(None)
    });

    s.bound_impl("::failure::Fail", quote! {
        fn fail(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self { #fail_body }
        }

        fn backtrace(&self) -> ::std::option::Option<&::failure::Backtrace> {
            match *self { #bt_body }
        }
    })
}

fn find_error_msg(attrs: &[syn::Attribute]) -> Result<&[syn::NestedMetaItem], ()> {
    let mut error_msg = Err(());
    for attr in attrs {
        if attr.name() == "error_msg" {
            if error_msg.is_ok() {
                // Two error_msg attributes
                error_msg = Err(());
                break
            } else {
                if let syn::MetaItem::List(_, ref list)  = attr.value {
                    error_msg = Ok(&list[..]);
                } else {
                    // error_msg is not a list attribute
                    error_msg = Err(());
                    break
                }
            }
        }
    }
    error_msg
}

fn is_backtrace(field: &syn::Field) -> bool {
    match field.ty {
        syn::Ty::Path(None, syn::Path { segments: ref path, .. }) => {
            path.last().map_or(false, |s| s.ident == "Backtrace" && s.parameters.is_empty())
        }
        _ => false
    }
}
