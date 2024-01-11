extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Signature, parse::{Parse, ParseStream}, Result, Token};

struct VecSignature {
    sigs: Vec<Signature>
}

impl Parse for VecSignature {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut sigs = vec![];
        while !input.is_empty() {
            let sig: Signature = input.parse()?;
            let _semi: Token![;] = input.parse()?;
            sigs.push(sig);
        }
        Ok(Self { sigs })
    }
}

fn import_impl_single(sig: &Signature) -> TokenStream {
    super::utils::verify_signature(&sig);
    let mut arg_names = vec![];
    for tok in sig.inputs.iter() {
        match tok.clone() {
            syn::FnArg::Receiver(_) => {
                // self, &self, &mut self are not allowed
                panic!();
            }
            syn::FnArg::Typed(pattype) => {
                let (pat, ty) = (*pattype.pat, *pattype.ty);
                arg_names.push(match pat {
                    syn::Pat::Ident(x) => { x.ident }
                    _ => { panic!() }
                });
                let _canonical_ty = super::utils::canonicalize_type(&ty);
            }
        }
    }
    let mangled = super::utils::mangle(sig);
    let basm_import_mod_mangled: TokenStream = ("basm_import_mod_".to_owned() + &mangled).parse().unwrap();
    let basm_import_mangled: TokenStream = ("basm_import_".to_owned() + &mangled).parse().unwrap();
    let fn_name = &sig.ident;
    let return_type: TokenStream = match &sig.output {
        syn::ReturnType::Default => { "()".parse().unwrap() }
        syn::ReturnType::Type(_x, y) => { quote!(#y) }
    };
    let out = quote! {
        mod #basm_import_mod_mangled {
            extern crate basm_std;
            use alloc::vec::Vec;
            use basm_std::serialization::{Ser, De, eat, Pair};
            use core::mem::transmute;
        
            static mut SER_VEC: Vec::<u8> = Vec::<u8>::new();
            static mut PTR_FN: usize = 0;
        
            #[cfg(target_arch = "x86_64")]
            #[inline(never)]
            unsafe extern "win64" fn free() { SER_VEC.clear() }
        
            #[cfg(not(target_arch = "x86_64"))]
            #[inline(never)]
            unsafe extern "C" fn free() { SER_VEC.clear() }
        
            #[cfg(target_arch = "x86_64")]
            #[no_mangle]
            #[inline(never)]
            unsafe extern "win64" fn #basm_import_mangled(ptr_fn: usize) { PTR_FN = ptr_fn; }
        
            #[cfg(not(target_arch = "x86_64"))]
            #[no_mangle]
            #[inline(never)]
            unsafe extern "C" fn #basm_import_mangled(ptr_fn: usize) { PTR_FN = ptr_fn; }
        
            pub #sig {
                unsafe {
                    #[cfg(target_arch = "x86_64")]
                    let ptr_fn: extern "win64" fn(usize) -> usize = transmute(PTR_FN);
                    #[cfg(not(target_arch = "x86_64"))]
                    let ptr_fn: extern "C" fn(usize) -> usize = transmute(PTR_FN);
            
                    assert!(SER_VEC.is_empty());
                    #( #arg_names.ser_len(&mut SER_VEC, 0); )*
                    (free as usize).ser(&mut SER_VEC);
                    let ptr_serialized = ptr_fn(SER_VEC.as_ptr() as usize);
            
                    let (mut buf, ptr_free_remote): (&'static [u8], usize) = eat(ptr_serialized);
                    #[cfg(target_arch = "x86_64")]
                    let free_remote: extern "win64" fn() -> () = transmute(ptr_free_remote);
                    #[cfg(not(target_arch = "x86_64"))]
                    let free_remote: extern "C" fn() -> () = transmute(ptr_free_remote);
        
                    let out = #return_type::de(&mut buf);
                    assert!(buf.is_empty());
                    free_remote();
                    out
                }
            }
        }
        use #basm_import_mod_mangled::#fn_name;
    };
    out
}

pub fn import_impl(input: TokenStream) -> TokenStream {
    let vecsig: VecSignature = syn::parse2(input).unwrap();
    let out: Vec<_> = vecsig.sigs.iter().map(|sig| {
        import_impl_single(sig)
    }).collect();
    quote! {
        #(#out)*
    }
}