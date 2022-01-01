use std::{collections::BTreeMap, rc::Rc};

use indexmap::IndexMap;
use proc_macro2::{Ident, Span, TokenStream};

use crate::gen::{CodeGenFn, FieldName, RustType, Type};

pub fn merge_struct(mut input: Vec<(Option<String>, Type)>) -> Type {
    // only one anon element in input => whatever the fuck the element is
    if input.len() == 1 && input.get(0).unwrap().0.is_none() {
        input.remove(0).1
        // check if input is valid for becoming a struct
    } else if input
        .iter()
        .filter(|(n, _)| n.is_none())
        .all(|(_, t)| matches!(t.rst, RustType::Struct(_)))
    {
        pub enum FnIndex {
            Named(String, CodeGenFn, RustType),
            Done(CodeGenFn),
        }

        let mut vec_of_ser = Vec::new();
        let mut vec_of_de = Vec::new();
        let mut vec_of_def = Vec::new();
        let mut rst_hm = BTreeMap::default();
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
}

pub fn struct_destructure(hm: BTreeMap<String, RustType>, struct_name: &FieldName) -> TokenStream {
    let mut ts = proc_macro2::TokenStream::default();
    let struct_ident = struct_name.to_type_ident();
    let ilvl = struct_name.get_ilvl();
    for (n, _) in hm {
        let struct_field_ident = Ident::new(&n, Span::call_site());
        let var_ident = Ident::new(&format!("{}{}", "_".repeat(ilvl), n), Span::call_site());
        ts.extend(quote::quote! {#struct_field_ident: #var_ident,});
    }
    quote::quote! {#struct_ident {#ts}}
}

pub fn option_unwrap(var_to_unwrap: &FieldName) -> TokenStream {
    let var_ident = var_to_unwrap.to_var_ident();
    quote::quote! {let #var_ident = #var_ident.unwrap();}
}

pub fn make_match(
    input: Vec<(TokenStream, Type)>,
    default: Option<Type>,
) -> Box<dyn FnOnce(Ident) -> Type> {
    let mut temp: IndexMap<RustType, Vec<(TokenStream, Type)>> = Default::default();

    macro_rules! insert {
        ($b:ident) => {
            if let Some(a) = temp.get_mut(&$b.1.rst) {
                a.push($b);
            } else {
                temp.insert($b.1.rst.to_owned(), vec![$b]);
            }
        };
    }

    input.into_iter().for_each(|a| insert!(a));

    let has_default = default.is_some();
    if let Some(default) = default {
        let a = (quote::quote! {_}, default);
        insert!(a);
    }

    let (mut rsts, struct_rsts): (Vec<RustType>, Vec<RustType>) = temp
        .iter()
        .map(|(a, _)| a)
        .cloned()
        .partition(|a| matches!(a, RustType::Struct(_)));

    let struct_rsts: Vec<BTreeMap<String, RustType>> = struct_rsts
        .into_iter()
        .map(|a| {
            if let RustType::Struct(a) = a {
                return a;
            };
            panic!()
        })
        .collect();
    if !struct_rsts.is_empty() {
        let struct_fields: Vec<_> = struct_rsts
            .iter()
            .map(|a| a.iter().map(|(a, _)| a.to_owned()).collect::<Vec<_>>())
            .reduce(|mut a, b| {
                for elem in b {
                    if !a.contains(&elem) {
                        a.push(elem)
                    }
                }
                a
            })
            .unwrap();
        let mut struct_rst = BTreeMap::new();
        for field in struct_fields {
            let mut field_rsts = vec![];
            for rst in &struct_rsts {
                let field_rst = match rst.get(&field) {
                    Some(rst) => rst.to_owned(),
                    None => RustType::None,
                };
                if !field_rsts.contains(&field_rst) {
                    field_rsts.push(field_rst);
                }
            }
            let rst = merge_vec_of_rst(field_rsts);
            struct_rst.insert(field, rst);
        }
        rsts.push(RustType::Struct(struct_rst))
    }
    let rst = merge_vec_of_rst(rsts);

    // match rst {
    //     RustType::Struct(_) => todo!(),
    //     RustType::Enum(_) => todo!(),
    //     RustType::Option(_) => todo!(),
    //     RustType::Vec(_) => todo!(),
    //     RustType::Array(_, _) => todo!(),
    //     RustType::Simple(_) => todo!(),
    //     RustType::None => todo!(),
    // }

    // let (rst, v) = temp.into_iter().next().unwrap();
    Box::new(move |match_ident| -> Type {
        Type {
            ser_code_gen_fn: Box::new(move |field_name| -> TokenStream {
                let enum_ident = field_name.to_type_ident();
                let match_var = field_name.to_var_ident();

                let mut matches_ts = TokenStream::default();
                for (matcher_ts, t) in v {
                    let destructuring_assignment = match t.rst {
                        RustType::Struct(hm) => {
                            let struct_destruct =
                                struct_destructure(hm, &"Struct".to_owned().into());
                            quote::quote! { #enum_ident :: #struct_destruct }
                        }
                        RustType::Option(rst) => {
                            let opt = option_unwrap(&field_name);
                            if let RustType::Struct(hm) = *rst {
                                let destructed_struct =
                                    struct_destructure(hm, &"Struct".to_owned().into());
                                quote::quote! {#opt #enum_ident :: #destructed_struct}
                            } else {
                                opt
                            }
                        }
                        _ => {
                            quote::quote! {}
                        }
                    };

                    let a = (t.ser_code_gen_fn)(field_name.to_owned());

                    matches_ts
                        .extend(quote::quote! {#matcher_ts => {#destructuring_assignment #a}});
                }

                quote::quote! {
                    match #match_ident {
                        #matches_ts
                    };
                }
            }),
            de_code_gen_fn: todo!(),
            def_code_gen_fn: todo!(),
            rst,
        }
    })
}

fn merge_vec_of_rst(field_rsts: Vec<RustType>) -> RustType {
    let wrap_in_opt = field_rsts.contains(&RustType::None);
    let mut field_rsts: Vec<_> = field_rsts
        .into_iter()
        .filter(|a| !matches!(a, RustType::None))
        .collect();
    if field_rsts.is_empty() {
        RustType::None
    } else {
        let ret = if field_rsts.len() == 1 {
            field_rsts.remove(0)
        } else {
            RustType::Enum(field_rsts)
        };
        if wrap_in_opt {
            RustType::Option(Box::new(ret))
        } else {
            ret
        }
    }
}
