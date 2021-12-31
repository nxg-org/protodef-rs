use std::collections::HashMap;

use convert_case::Case;
use proc_macro2::{Ident, Span};

use crate::gen::{CodeGenFn, FieldName, RustType, Type};

pub fn merge(mut input: Vec<(Option<FieldName>, Type)>) -> Type {
    // either all none
    // or only none with rusttype struct
    input.iter().for_each(|(n, t)| {
        if let Some(n) = n {
            println!("{:?}", (n.to_case(Case::Snake), &t.rst))
        } else {
            println!("{:?}", &t.rst)
        }
    });
    // only one anon element in input => whatever the fuck the element is
    if input.len() == 1 && input.get(0).unwrap().0.is_none() {
        println!("only one anon elem in input");
        input.remove(0).1
    // only anonymous elements in input => enum
    } else if input.iter().all(|(n, _)| n.is_none()) {
        todo!("imma make an enum from that")
    // only named elements and anonymous structs in input => struct
    } else if input
        .iter()
        .filter(|(n, _)| n.is_none())
        .all(|(_, t)| matches!(t.rst, RustType::Struct(_)))
    {
        println!("making a struct from this");
        pub enum FnIndex {
            Named(FieldName, CodeGenFn),
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
                vec_of_ser.push(FnIndex::Named(field_name.clone(), ser_code_gen_fn));
                vec_of_de.push(FnIndex::Named(field_name.clone(), de_code_gen_fn));
                vec_of_def.push(FnIndex::Named(field_name.clone(), def_code_gen_fn));
                rst_hm.insert(field_name.to_case(Case::Snake), rst);
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
            ser_code_gen_fn: Box::new(move |field_name_1, ilvl| -> proc_macro2::TokenStream {
                let mut ts = proc_macro2::TokenStream::default();
                for fni in vec_of_ser {
                    match fni {
                        FnIndex::Named(field_name_2, f) => ts.extend(f(field_name_2, ilvl + 1)),
                        FnIndex::Done(f) => {
                            let done_code = f(field_name_1.clone(), ilvl);
                            ts.extend(quote::quote! {#done_code;})
                        }
                    };
                }
                quote::quote! {{#ts}}
            }),
            de_code_gen_fn: Box::new(move |field_name_1, ilvl| -> proc_macro2::TokenStream {
                let mut ts = proc_macro2::TokenStream::default();
                for fni in vec_of_de {
                    match fni {
                        FnIndex::Named(field_name_2, f) => {
                            let field_ident = Ident::new(
                                &format!(
                                    "{}{}",
                                    "_".repeat(ilvl + 1),
                                    field_name_2.to_case(Case::Snake)
                                ),
                                Span::call_site(),
                            );
                            let output = f(field_name_2, ilvl + 1);
                            ts.extend(quote::quote! {let #field_ident = #output;});
                        }
                        FnIndex::Done(f) => {
                            let done_code = f(field_name_1.clone(), ilvl);
                            ts.extend(quote::quote! {#done_code;})
                        }
                    }
                }
                quote::quote! {{#ts}}
            }),
            def_code_gen_fn: Box::new(move |field_name_1, ilvl| -> proc_macro2::TokenStream {
                let mut ts = proc_macro2::TokenStream::default();
                for fni in vec_of_def {
                    match fni {
                        FnIndex::Named(field_name_2, f) => ts.extend(f(field_name_2, ilvl + 1)),
                        FnIndex::Done(f) => ts.extend(f(field_name_1.clone(), ilvl)),
                    }
                }
                ts
            }),
            rst: RustType::Struct(rst_hm),
        }

        // def functions merging:

        //     let mut code_gen_fn_in_order = Vec::new();
        // let mut structs_merged_down = Vec::new();
        // for (n, elem) in input {
        //     code_gen_fn_in_order.push();
        //     match (n, elem.rst) {
        //         (None, RustType::Struct(hm)) => {
        //             for (n, t) in hm {
        //                 structs_merged_down.push((Some(n), t))
        //             }
        //         }
        //         (n, t) => structs_merged_down.push((n, t)),
        //     };
        // }

        // return Type{
        //     rst,
        //     ser_code_gen_fn: Box::new(|field_name| {
        //         todo!()
        //     }),
        //     de_code_gen_fn: Box::new(|field_name|{
        //         todo!()
        //     }),
        //     def_code_gen_fn: Box::new(|field_name|{
        //         todo!()
        //     }),
        // }
        // todo!("imma make a struct from that")
    } else {
        panic!("passed something to merge that can neither be put into a struct nor an enum and isn't a single simple type")
    }
    // todo!()
    // let mut code_gen_fn_in_order = Vec::new();
    // let mut structs_merged_down = Vec::new();
    // for (n, elem) in input {
    //     code_gen_fn_in_order.push(elem.code_gen_fn);
    //     match (n, elem.rst) {
    //         (None, RustType::Struct(hm)) => {
    //             for (n, t) in hm {
    //                 structs_merged_down.push((Some(n), t))
    //             }
    //         }
    //         (n, t) => structs_merged_down.push((n, t)),
    //     };
    // }

    /*
        [
            (None, Simple("u64")),
            (None, Enum([
                Simple("u64"),
                Struct({"a": "u64"})
            ])),
        ]

        [
            "container",
            [
                {
                    "anon": true,
                    "type": "u64"
                },
                {
                    "anon": true,
                    "type": [
                        "switch",
                        {
                            "fields": {
                                "0x00": "u64",
                                "0x01": [
                                    "container",
                                    [
                                        {
                                            "name": "a",
                                            "type": "u64"
                                        }
                                    ]
                                ]
                            }
                        }
                    ]
                }
            ]
        ]



        [
            "container",
            [
                {
                    "anon": true,
                    "type": "u64"
                },
                {
                    "name": "b",
                    "type": "u8"
                }
            ]
        ]
            (None, Simple("u8")),
            (None, Struct({
                "a":"u64"
            })),
            (Some("b"), Simple("u64"))

        [
            "switch",
            {
                "compareTo": "switch_compareto_something_something",
                "fields": {
                    "0x00": "u8",
                    "0x01": "u64",
                    "0x02": [
                        "container",
                        [
                            {
                                "name": "a",
                                "type": "u64"
                            }
                        ]
                    ]
                }
            }
        ]

        [
            "container",
            [
                {
                    anon: true,
                    type: "u64"
                },
                {
                    anon: true,
                    type: "u8",
                },
                {

                }
            ]
        ]
    */

    // todo!()

    // let rusttypes = input
    //     .iter()
    //     .map(|(n, t)| (n.to_owned(), t.rst.to_owned()))
    //     .collect::<Vec<_>>();
    // let contains_root_singles = rusttypes
    //     .iter()
    //     .filter(|(n, _)| n.is_none())
    //     .any(|(_, t)| matches!(t, RustType::Simple(_) | RustType::Enum(_)));
    // println!("contains root singles: {}", contains_root_singles);
    // return Type{ code_gen_fn: Box::new(|_|panic!()), rst: RustType::Simple("fuck you".to_owned()) };

    // {
    //     Some("name"): Simple("string"),
    //     Some("b"): Simple("u64")
    //     None: Struct({
    //         c: "u8"
    //     })
    // }

    // {
    //     name: String,
    //     b: u64,
    //     c: u8
    // }

    // todo!("actually think of logic");
    // // let (named_variants, root_variants): (Vec<(String, Type)>, Vec<Type>) = ;

    // let mut reduced_root_variants = Vec::new();
    // for (name, r#type) in input.iter() {
    //     if name.is_none() && !reduced_root_variants.contains(&r#type.rst) {
    //         reduced_root_variants.push(r#type.rst.to_owned());
    //     }
    // }

    // let rst = if input.len() == 1 && input.iter().all(|(n, _)| n.is_none()) {
    //     let (_, t) = input.get(0).unwrap();
    //     t.rst.clone()
    // } else if input
    //     .iter()
    //     .all(|(n, t)| n.is_some() || matches!(t.rst, RustType::Struct(_)))
    // {
    //     // merge down nested anonymous structs
    //     let mut hm = HashMap::new();
    //     for (name, t) in input {
    //         match name {
    //             Some(name) => {
    //                 hm.insert(name, t.rst);
    //             }
    //             None => {
    //                 if let RustType::Struct(shm) = t.rst {
    //                     for (sub_field_name, sub_field_type) in shm {
    //                         hm.insert(sub_field_name, sub_field_type);
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     RustType::Struct(hm)
    // } else {
    //     todo!()
    // };

    // match rst {
    //     RustType::Struct(_) => todo!(),
    //     RustType::Enum(_) => todo!(),
    //     RustType::Simple(_) => todo!(),
    // }

    // input.iter().for_each(|(name, t)| {});
    // Type {
    //     code_gen_fn: todo!(),
    //     rst: todo!(),
    // }
}

/*














































































































































































































































































































































































































*/
