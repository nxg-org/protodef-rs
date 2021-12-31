use std::collections::HashMap;
pub mod container;
pub mod switch;
use crate::native::{container::native_container, switch::native_switch};

use super::gen::*;
use proc_macro2::TokenStream;
use quote::quote;
use serde_json::Value;

pub type Natives = HashMap<String, TypeFunction>;

lazy_static::lazy_static!(
    pub static ref BUILTIN_NATIVES: Natives = {
        let mut ret = Natives::default();

        macro_rules! simple_natives {
            ($($native_name:expr => $native_fun_name:ident $buf_getter:ident $buf_putter:ident $rstype:expr)+) => {
                $(
                    fn $native_fun_name(
                        _: TypeFunctionContext,
                        opts: Option<&Value>
                    ) -> GetGenTypeResult {
                        if opts.is_some() {
                            panic!("you cannot pass arguments to a {}", $rstype)
                        }
                        GetGenTypeResult::Done(Type {
                            rst: RustType::Simple($rstype.to_owned()),
                            ser_code_gen_fn: Box::new(|field_name| -> TokenStream {
                                let field_ident = field_name.to_var_ident();
                                quote! {buf.$buf_putter(#field_ident);}
                            }),
                            de_code_gen_fn: Box::new(|_| -> TokenStream {
                                quote! {buf.$buf_getter()}
                            }),
                            def_code_gen_fn: Box::new(|_| -> TokenStream {
                                quote! {}
                            }),
                        })
                    }
                    ret.insert($native_name.to_owned(), Box::new(&$native_fun_name));
                )+
            }
        }

        simple_natives!(
            "varint" => native_varint get_var_int put_var_int "i32"
            "varlong" => native_varlong get_var_long put_var_long "i64"
            "u8"  => native_u8 get_u8 put_u8 "u8"
            "i8"  => native_i8 get_i8 put_i8 "i8"
            "u16" => native_u16 get_u16 put_u16 "u16"
            "i16" => native_i16 get_i16 put_i16 "i16"
            "u32" => native_u32 get_u32 put_u32 "u32"
            "i32" => native_i32 get_i32 put_i32 "i32"
            "u64" => native_u64 get_u64 put_u64 "u64"
            "i64" => native_i64 get_i64 put_i64 "i64"
            "u128" => native_u128 get_u128 put_u128 "u128"
            "i128" => native_i128 get_i128 put_i128 "i128"
            "f32" => native_f32 get_f32 put_f32 "f32"
            "f64" => native_f64 get_f64 put_f64 "f64"
            "boolean" => native_bool get_bool put_bool "bool"
        );

        ret.insert("switch".to_owned(), Box::new(&native_switch));
        ret.insert("container".to_owned(), Box::new(&native_container));

        ret
    };
);
// fn $native_fun_name(
//     _: TypeFunctionContext,
//     _: Option<&Value>
// ) -> GetGenTypeResult {
//     GetGenTypeResult::Done(Type {
//         code_gen_fn: Box::new(|field_name| -> OutCode {
//             let ident = proc_macro2::Ident::new(&field_name, proc_macro2::Span::call_site());

//             OutCode {
//                 ser: quote! {buf.$buf_putter(#ident)},
//                 de: quote! {buf.$buf_getter();},
//                 def: quote! {},
//             }
//         }),
//         rst: RustType::Simple($rstype.to_owned()),
//     })
// }
// ret.insert($rstype, Box::new(&$native_fun_name));
// fn native_u64(ctx: TypeFunctionContext, opts: Option<&Value>) -> GetGenTypeResult {
//     GetGenTypeResult::Done(Type {
//         rst: RustType::Simple("u64".to_owned()),
//         ser_code_gen_fn: Box::new(|field_name| -> TokenStream {
//             let field_ident = field_name.to_var_ident();
//             quote! {buf.put_u64(#field_ident)}
//         }),
//         de_code_gen_fn: Box::new(|field_name| -> TokenStream {
//             quote! {buf.get_u64()}
//         }),
//         def_code_gen_fn: Box::new(|field_name| -> TokenStream {
//             let type_ident = field_name.to_type_ident();
//             quote! {pub type #type_ident = u64}
//         }),
//     })
// }
// // ret.insert("u64", Box::new(&native_u64));
