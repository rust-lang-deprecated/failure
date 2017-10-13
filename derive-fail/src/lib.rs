extern crate proc_macro;
extern crate syn;

#[macro_use] extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(Fail, attributes(error_msg))]
pub fn derive_fail(input: TokenStream) -> TokenStream {
    let ast = syn::parse_derive_input(&input.to_string()).unwrap();
    let gen = impl_fail(&ast).unwrap();
    gen.parse().unwrap()
}

fn impl_fail(ast: &syn::DeriveInput) -> Result<quote::Tokens, ()> {
    match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => struct_fail(ast, fields),
        syn::Body::Struct(syn::VariantData::Unit)               => unit_fail(ast),
        syn::Body::Struct(syn::VariantData::Tuple(_))           => tuple_fail(ast),
        syn::Body::Enum(ref variants)                           => enum_fail(ast, variants),
    }
}

fn unit_fail(ast: &syn::DeriveInput) -> Result<quote::Tokens, ()> {
    let ty = &ast.ident;
    let generics = &ast.generics;
    let error_msg = {
        let list = find_error_msg(&ast.attrs)?;
        unit_error_msg(list)?
    };
    Ok(quote! {
        impl<#generics> ::failure::Fail for #ty<#generics> {
            fn fail(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                #error_msg
            }

        }
    })
}

fn struct_fail(ast: &syn::DeriveInput, fields: &[syn::Field]) -> Result<quote::Tokens, ()> {
    let ty = &ast.ident;
    let generics = &ast.generics;
    let error_msg = {
        let list = find_error_msg(&ast.attrs)?;
        struct_error_msg(fields, list)
    };
    let backtrace = backtrace(ast);
    Ok(quote! {
        impl<#generics> ::failure::Fail for #ty<#generics> {
            #[allow(unused_variables)]
            fn fail(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                #error_msg
            }

            #backtrace
        }
    })
}

fn struct_error_msg(fields: &[syn::Field], meta_items: &[syn::NestedMetaItem]) -> quote::Tokens {
    let fields = fields.iter().filter_map(|field| field.ident.as_ref());
    quote! {
        {
            let Self { #(ref #fields,)* } = *self;
            write!(f, #(#meta_items,)*)
        }
    }
}

fn tuple_fail(ast: &syn::DeriveInput) -> Result<quote::Tokens, ()> {
    let ty = &ast.ident;
    let generics = &ast.generics;
    let error_msg = {
        let list = find_error_msg(&ast.attrs)?;
        tuple_error_msg(list)
    };
    let backtrace = backtrace(ast);
    Ok(quote! {
        impl<#generics> ::failure::Fail for #ty<#generics> {
            #[allow(unused_variables)]
            fn fail(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                #error_msg
            }

            #backtrace
        }
    })
}

fn enum_fail(ast: &syn::DeriveInput, variants: &[syn::Variant]) -> Result<quote::Tokens, ()> {
    let ty = &ast.ident;
    let generics = &ast.generics;
    let error_msg = enum_error_msg(ty, variants)?;
    let backtrace = enum_backtrace(ty, variants);
    Ok(quote! {
        impl<#generics> ::failure::Fail for #ty<#generics> {
            #[allow(unused_variables)]
            fn fail(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                #error_msg
            }

            #backtrace
        }
    })
}

fn enum_error_msg(ty: &syn::Ident, variants: &[syn::Variant]) -> Result<quote::Tokens, ()> {
    let error_msgs = variants.iter().map(|variant| {
        let meta_items = find_error_msg(&variant.attrs)?;
        match variant.data {
            syn::VariantData::Struct(ref fields)    => {
                let ident = &variant.ident;
                let fields = fields.iter().filter_map(|field| field.ident.as_ref());
                Ok(quote!(#ty::#ident { #(ref #fields,)* } => {
                    write!(f, #(#meta_items,)*)
                }))
            }
            syn::VariantData::Tuple(ref fields)     => {
                let ident = &variant.ident;
                let fields = fields.iter().enumerate().map(|(n, _)| {
                    let int: syn::Ident = format!("_{}", n).into();
                    quote!(#int)
                });
                let meta_items = meta_items.iter().map(|meta_item| match *meta_item {
                    syn::NestedMetaItem::Literal(syn::Lit::Int(n, _)) => {
                        let int: syn::Ident = format!("_{}", n).into();
                        quote!(#int)
                    }
                    _   => quote!(#meta_item),
                });
                Ok(quote!(
                    #ty::#ident(#(ref #fields,)*) => { write!(f, #(#meta_items,)*) }
                ))
            }
            syn::VariantData::Unit                  => {
                let ident = &variant.ident;
                let tokens = unit_error_msg(meta_items)?;
                Ok(quote!( #ty::#ident => #tokens ))
            }
        }
    }).collect::<Result<Vec<_>, _>>()?;
    Ok(quote!(
        match *self {
            #(#error_msgs)*
        }
    ))
}

fn tuple_error_msg(meta_items: &[syn::NestedMetaItem]) -> quote::Tokens {
    let list = meta_items.iter().map(|meta_item| match *meta_item {
        ref int @ syn::NestedMetaItem::Literal(syn::Lit::Int(_, syn::IntTy::Unsuffixed)) => {
            quote!(self.#int)
        }
        _   => quote!(#meta_item),
    });
    quote!({ write!(f, #(#list,)*) })
}

fn unit_error_msg(meta_items: &[syn::NestedMetaItem]) -> Result<quote::Tokens, ()> {
    if meta_items.len() != 1 { return Err(()) }
    match *meta_items.first().unwrap() {
        syn::NestedMetaItem::Literal(syn::Lit::Str(ref s, ..))  => Ok(quote!({ f.write_str(#s) })),
        _                                                       => Err(()),
    }
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

fn enum_backtrace(ty: &syn::Ident, variants: &[syn::Variant]) -> quote::Tokens {
    let backtraces = variants.iter().map(|variant| {
        match variant.data {
            syn::VariantData::Struct(ref fields)    => {
                let ident = &variant.ident;
                match find_backtrace_field(fields) {
                    Some(btrace)    => {
                        let btrace = btrace.ident.as_ref().unwrap();
                        quote!( #ty::#ident { ref #btrace, .. } => { Some(#btrace) })
                    }
                    None            => {
                        quote!( #ty::#ident { .. } => { None })
                    }
                }
            }
            syn::VariantData::Tuple(ref fields)     => {
                let ident = &variant.ident;
                match find_backtrace_field_tuple(fields) {
                    Some((idx, _))    => {
                        let null_fields = (0..idx).map(|_| quote!(_));
                        let btrace = syn::Ident::new("btrace");
                        quote!( #ty::#ident(#(#null_fields,)* ref #btrace, ..) => { Some(#btrace) })
                    }
                    None            => {
                        quote!( #ty::#ident(..) => { None })
                    }
                }
            }
            syn::VariantData::Unit                  => {
                let ident = &variant.ident;
                quote!( #ty::#ident => { None })
            }
        }
    });
    quote!(
        fn backtrace(&self) -> Option<&failure::Backtrace> {
            match *self {
                #(#backtraces)*
            }
        }
    )
}

fn backtrace(ast: &syn::DeriveInput) -> Option<quote::Tokens> {
    let field = match ast.body {
        syn::Body::Struct(syn::VariantData::Struct(ref fields)) => {
            find_backtrace_field(fields).and_then(|field| field.ident.clone())
        }
        syn::Body::Struct(syn::VariantData::Tuple(ref fields)) => {
            find_backtrace_field_tuple(fields).map(|(idx, _)| idx.into())
        }
        _ => None,
    };

    field.map(|field| quote! {
        fn backtrace(&self) -> Option<&failure::Backtrace> {
            Some(&self.#field)
        }
    })
}

fn find_backtrace_field(fields: &[syn::Field]) -> Option<&syn::Field> {
    fields.iter().find(|field| is_backtrace(field))
}

fn find_backtrace_field_tuple(fields: &[syn::Field]) -> Option<(usize, &syn::Field)> {
    fields.iter().enumerate().find(|&(_, field)| is_backtrace(&field))
}

fn is_backtrace(field: &syn::Field) -> bool {
    match field.ty {
        syn::Ty::Path(None, syn::Path { segments: ref path, .. }) => {
            path.last().map_or(false, |s| s.ident == "Backtrace" && s.parameters.is_empty())
        }
        _ => false
    }
}
