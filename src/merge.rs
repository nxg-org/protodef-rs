use std::{collections::HashMap, rc::Rc};

use proc_macro2::{Ident, Span, TokenStream};

use crate::gen::{CodeGenFn, FieldName, RustType, Type};

pub fn merge_struct(mut input: Vec<(Option<String>, Type)>) -> Type {
    // either all none
    // or only none with rusttype struct
    // only one anon element in input => whatever the fuck the element is
    if input.len() == 1 && input.get(0).unwrap().0.is_none() {
        // println!("only one anon elem in input");
        input.remove(0).1
    // only anonymous elements in input => enum
    } else if input
        .iter()
        .filter(|(n, _)| n.is_none())
        .all(|(_, t)| matches!(t.rst, RustType::Struct(_)))
    {
        // println!("making a struct from this");
        pub enum FnIndex {
            Named(String, CodeGenFn, RustType),
            Done(CodeGenFn),
        }

        let mut vec_of_ser = Vec::new();
        let mut vec_of_de = Vec::new();
        let mut vec_of_def = Vec::new();
        let mut rst_hm = HashMap::default();
        for (n, t) in input {
            let Type {
                ser_code_gen_fn,
                de_code_gen_fn,
                def_code_gen_fn,
                rst,
            } = t;
            if let Some(field_name) = n {
                vec_of_ser.push(FnIndex::Named(
                    field_name.clone(),
                    ser_code_gen_fn,
                    rst.to_owned(),
                ));
                vec_of_de.push(FnIndex::Named(
                    field_name.clone(),
                    de_code_gen_fn,
                    rst.to_owned(),
                ));
                vec_of_def.push(FnIndex::Named(
                    field_name.clone(),
                    def_code_gen_fn,
                    rst.to_owned(),
                ));
                rst_hm.insert(field_name, rst);
            } else if let RustType::Struct(hm) = rst {
                vec_of_ser.push(FnIndex::Done(ser_code_gen_fn));
                vec_of_de.push(FnIndex::Done(de_code_gen_fn));
                vec_of_def.push(FnIndex::Done(def_code_gen_fn));
                for (n, rst) in hm {
                    rst_hm.insert(n, rst);
                }
            } else {
                panic!("that is fucking impossible bro, you're never gonna reach me!")
            }
        }
        Type {
            ser_code_gen_fn: Box::new(move |field_name_1| -> proc_macro2::TokenStream {
                let mut ts = proc_macro2::TokenStream::default();
                for fni in vec_of_ser {
                    match fni {
                        FnIndex::Named(field_name_2, f, rst) => {
                            let field_name = field_name_1.push(Box::new(Rc::new(field_name_2)));
                            let struct_destructure = match rst {
                                RustType::Struct(hm) => {
                                    let struct_destructure = struct_destructure(hm, &field_name);
                                    let struct_var_ident = field_name.to_var_ident();
                                    quote::quote! {let #struct_destructure = #struct_var_ident;}
                                }
                                _ => proc_macro2::TokenStream::default(),
                            };
                            let out_code = f(field_name);
                            ts.extend(quote::quote! {#struct_destructure #out_code})
                        }
                        FnIndex::Done(f) => {
                            let done_code = f(field_name_1.clone());
                            ts.extend(quote::quote! {#done_code})
                        }
                    };
                }
                quote::quote! {#ts}
            }),
            de_code_gen_fn: Box::new(move |field_name_1| -> proc_macro2::TokenStream {
                let mut ts = proc_macro2::TokenStream::default();
                for fni in vec_of_de {
                    match fni {
                        FnIndex::Named(field_name_2, f, rst) => {
                            let field_name = field_name_1.push(Box::new(Rc::new(field_name_2)));
                            let field_ident = field_name.to_var_ident();
                            let struct_destructure = match rst {
                                RustType::Struct(hm) => struct_destructure(hm, &field_name),
                                _ => proc_macro2::TokenStream::default(),
                            };
                            let output = f(field_name);
                            ts.extend(
                                quote::quote! {let #field_ident = {#output #struct_destructure};},
                            );
                        }
                        FnIndex::Done(f) => {
                            let done_code = f(field_name_1.clone());
                            ts.extend(quote::quote! {#done_code})
                        }
                    }
                }
                quote::quote! {#ts}
            }),
            def_code_gen_fn: Box::new(move |field_name_1| -> proc_macro2::TokenStream {
                let mut ts = proc_macro2::TokenStream::default();
                for fni in vec_of_def {
                    match fni {
                        FnIndex::Named(field_name_2, f, _rst) => {
                            let field_name = field_name_1.push(Box::new(Rc::new(field_name_2)));
                            // let mut a = field_name_1.push(field_name_2);
                            ts.extend(f(field_name));
                        }
                        FnIndex::Done(f) => ts.extend(f(field_name_1.clone())),
                    }
                }
                ts
            }),
            rst: RustType::Struct(rst_hm),
        }
    } else {
        panic!("passed something to merge that can neither be put into a struct nor an enum and isn't a single simple type")
    }
    // todo!()
}

pub fn struct_destructure(hm: HashMap<String, RustType>, struct_name: &FieldName) -> TokenStream {
    let mut ts = proc_macro2::TokenStream::default();
    let struct_ident = struct_name.to_type_ident();
    let ilvl = struct_name.get_ilvl();
    for (n, _) in hm {
        let struct_ident = Ident::new(&n, Span::call_site());
        let var_ident = Ident::new(&format!("{}{}", "_".repeat(ilvl), n), Span::call_site());
        ts.extend(quote::quote! {#struct_ident: #var_ident,});
    }
    quote::quote! {#struct_ident {#ts}}
}
