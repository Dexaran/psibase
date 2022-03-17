use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Fields};

struct Field<'a> {
    name: &'a proc_macro2::Ident,
    ty: &'a syn::Type,
}

fn struct_fields(data: &DataStruct) -> Vec<Field> {
    match &data.fields {
        Fields::Named(named) => named
            .named
            .iter()
            .map(|field| Field {
                name: field.ident.as_ref().unwrap(),
                ty: &field.ty,
            })
            .collect(),
        Fields::Unnamed(_) => unimplemented!("fracpack does not support unnamed fields"),
        Fields::Unit => unimplemented!("fracpack does not support unit struct"),
    }
}

fn enum_fields(data: &DataEnum) -> Vec<Field> {
    data.variants
        .iter()
        .map(|var| match &var.fields {
            Fields::Named(_) => unimplemented!("variants must have exactly 1 unnamed field"),
            Fields::Unnamed(field) => {
                assert!(
                    field.unnamed.len() == 1,
                    "variants must have exactly 1 unnamed field"
                );
                Field {
                    name: &var.ident,
                    ty: &field.unnamed[0].ty,
                }
            }
            Fields::Unit => unimplemented!("variants must have exactly 1 unnamed field"),
        })
        .collect()
}

pub fn fracpack_macro_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match &input.data {
        Data::Struct(data) => process_struct(&input, data),
        Data::Enum(data) => process_enum(&input, data),
        Data::Union(_) => unimplemented!("fracpack does not support union"),
    }
}

// TODO: compile time: verify no non-optionals are after an optional
// TODO: unpack: check optionals not in heap
fn process_struct(input: &DeriveInput, data: &DataStruct) -> TokenStream {
    let name = &input.ident;
    let generics = &input.generics;
    let fields = struct_fields(data);
    let fixed_size = fields
        .iter()
        .map(|field| {
            let ty = &field.ty;
            quote! {<#ty>::FIXED_SIZE}
        })
        .fold(quote! {0}, |acc, new| quote! {#acc + #new});
    let positions: Vec<syn::Ident> = fields
        .iter()
        .map(|field| {
            let name = &field.name;
            let concatenated = format!("pos_{}", name);
            syn::Ident::new(&concatenated, name.span())
        })
        .collect();
    let pack_fixed_members = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let name = &field.name;
            let pos = &positions[i];
            quote! {
                let #pos = dest.len() as u32;
                self.#name.embedded_fixed_pack(dest);
            }
        })
        .fold(quote! {}, |acc, new| quote! {#acc #new});
    let pack_variable_members = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let name = &field.name;
            let pos = &positions[i];
            quote! {
                self.#name.embedded_fixed_repack(#pos, dest.len() as u32, dest);
                self.#name.embedded_variable_pack(dest);
            }
        })
        .fold(quote! {}, |acc, new| quote! {#acc #new});
    let unpack = fields
        .iter()
        .map(|field| {
            let name = &field.name;
            let ty = &field.ty;
            quote! {
                #name: <#ty>::embedded_unpack(src, pos, &mut heap_pos)?,
            }
        })
        .fold(quote! {}, |acc, new| quote! {#acc #new});
    // TODO: skip unknown members
    // TODO: option to verify no unknown members
    let verify = fields
        .iter()
        .map(|field| {
            let ty = &field.ty;
            quote! {
                <#ty>::embedded_verify(src, pos, &mut heap_pos)?;
            }
        })
        .fold(quote! {}, |acc, new| quote! {#acc #new});
    TokenStream::from(quote! {
        impl fracpack::Packable for #name #generics {
            const FIXED_SIZE: u32 = 4;

            fn pack(&self, dest: &mut Vec<u8>) {
                let heap = #fixed_size;
                assert!(heap as u16 as u32 == heap);
                (heap as u16).pack(dest);
                #pack_fixed_members
                #pack_variable_members
            }
            fn unpack(src: &[u8], pos: &mut u32) -> fracpack::Result<Self> {
                let heap_size = u16::unpack(src, pos)?;
                let mut heap_pos = *pos + heap_size as u32;
                if heap_pos < *pos {
                    return Err(fracpack::Error::BadOffset);
                }
                let result = Self {
                    #unpack
                };
                *pos = heap_pos;
                Ok(result)
            }
            fn verify(src: &[u8], pos: &mut u32) -> fracpack::Result<()> {
                let heap_size = u16::unpack(src, pos)?;
                let mut heap_pos = *pos + heap_size as u32;
                if heap_pos < *pos {
                    return Err(fracpack::Error::BadOffset);
                }
                #verify
                *pos = heap_pos;
                Ok(())
            }
            fn embedded_fixed_pack(&self, dest: &mut Vec<u8>) {
                dest.extend_from_slice(&0_u32.to_le_bytes());
            }
            fn embedded_fixed_repack(&self, fixed_pos: u32, heap_pos: u32, dest: &mut Vec<u8>) {
                dest[fixed_pos as usize..fixed_pos as usize + 4]
                    .copy_from_slice(&(heap_pos - fixed_pos).to_le_bytes());
            }
            fn embedded_variable_pack(&self, dest: &mut Vec<u8>) {
                self.pack(dest)
            }
            fn embedded_unpack(src: &[u8], fixed_pos: &mut u32, heap_pos: &mut u32) -> fracpack::Result<Self> {
                let orig_pos = *fixed_pos;
                let offset = u32::unpack(src, fixed_pos)?;
                if *heap_pos as u64 != orig_pos as u64 + offset as u64 {
                    return Err(fracpack::Error::BadOffset);
                }
                Self::unpack(src, heap_pos)
            }
            fn embedded_verify(src: &[u8], fixed_pos: &mut u32, heap_pos: &mut u32) -> fracpack::Result<()> {
                let orig_pos = *fixed_pos;
                let offset = u32::unpack(src, fixed_pos)?;
                if *heap_pos as u64 != orig_pos as u64 + offset as u64 {
                    return Err(fracpack::Error::BadOffset);
                }
                Self::verify(src, heap_pos)
            }
            fn option_fixed_pack(opt: &Option<Self>, dest: &mut Vec<u8>) {
                match opt {
                    Some(x) => x.embedded_fixed_pack(dest),
                    None => dest.extend_from_slice(&1u32.to_le_bytes()),
                }
            }
            fn option_fixed_repack(opt: &Option<Self>, fixed_pos: u32, heap_pos: u32, dest: &mut Vec<u8>) {
                match opt {
                    Some(x) => x.embedded_fixed_repack(fixed_pos, heap_pos, dest),
                    None => (),
                }
            }
            fn option_variable_pack(opt: &Option<Self>, dest: &mut Vec<u8>) {
                match opt {
                    Some(x) => x.embedded_variable_pack(dest),
                    None => (),
                }
            }
            fn option_unpack(src: &[u8], fixed_pos: &mut u32, heap_pos: &mut u32) -> fracpack::Result<Option<Self>> {
                let offset = u32::unpack(src, fixed_pos)?;
                if offset == 1 {
                    return Ok(None);
                }
                *fixed_pos -= 4;
                Ok(Some(Self::embedded_unpack(src, fixed_pos, heap_pos)?))
            }
            fn option_verify(src: &[u8], fixed_pos: &mut u32, heap_pos: &mut u32) -> fracpack::Result<()> {
                let offset = u32::unpack(src, fixed_pos)?;
                if offset == 1 {
                    return Ok(());
                }
                *fixed_pos -= 4;
                Self::embedded_verify(src, fixed_pos, heap_pos)
            }
        }
    })
} // process_struct

fn process_enum(input: &DeriveInput, data: &DataEnum) -> TokenStream {
    let name = &input.ident;
    let generics = &input.generics;
    let fields = enum_fields(data);
    assert!(fields.len() < 256);
    let pack_items = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let index = i as u8;
            let field_name = &field.name;
            quote! {#name::#field_name(x) => {
                dest.push(#index);
                size_pos = dest.len();
                dest.extend_from_slice(&0_u32.to_le_bytes());
                x.pack(dest);
            }}
        })
        .fold(quote! {}, |acc, new| quote! {#acc #new});
    let unpack_items = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let index = i as u8;
            let field_name = &field.name;
            let ty = &field.ty;
            quote! {
                #index => #name::#field_name(<#ty>::unpack(src, pos)?),
            }
        })
        .fold(quote! {}, |acc, new| quote! {#acc #new});
    let verify_items = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let index = i as u8;
            let ty = &field.ty;
            quote! {
                #index => <#ty>::verify(src, pos)?,
            }
        })
        .fold(quote! {}, |acc, new| quote! {#acc #new});
    TokenStream::from(quote! {
        impl fracpack::Packable for #name #generics {
            const FIXED_SIZE: u32 = 4;

            fn pack(&self, dest: &mut Vec<u8>) {
                let size_pos;
                match &self {
                    #pack_items
                };
                let size = (dest.len() - size_pos - 4) as u32;
                dest[size_pos..size_pos + 4]
                    .copy_from_slice(&size.to_le_bytes());
            }
            fn unpack(src: &[u8], pos: &mut u32) -> fracpack::Result<Self> {
                let index = u8::unpack(src, pos)?;
                let size_pos = *pos;
                let size = u32::unpack(src, pos)?;
                let result = match index {
                    #unpack_items
                    _ => return Err(fracpack::Error::BadEnumIndex)
                };
                if *pos != size_pos + 4 + size {
                    return Err(fracpack::Error::BadSize);
                }
                Ok(result)
            }
            // TODO: option to error on unknown index
            fn verify(src: &[u8], pos: &mut u32) -> fracpack::Result<()> {
                let index = u8::unpack(src, pos)?;
                let size_pos = *pos;
                let size = u32::unpack(src, pos)?;
                match index {
                    #verify_items
                    _ => {
                        *pos = size_pos + 4 + size;
                        return Ok(())
                    },
                }
                if *pos != size_pos + 4 + size {
                    return Err(fracpack::Error::BadSize);
                }
                Ok(())
            }
            fn embedded_fixed_pack(&self, dest: &mut Vec<u8>) {
                dest.extend_from_slice(&0_u32.to_le_bytes());
            }
            fn embedded_fixed_repack(&self, fixed_pos: u32, heap_pos: u32, dest: &mut Vec<u8>) {
                dest[fixed_pos as usize..fixed_pos as usize + 4]
                    .copy_from_slice(&(heap_pos - fixed_pos).to_le_bytes());
            }
            fn embedded_variable_pack(&self, dest: &mut Vec<u8>) {
                self.pack(dest)
            }
            fn embedded_unpack(src: &[u8], fixed_pos: &mut u32, heap_pos: &mut u32) -> fracpack::Result<Self> {
                let orig_pos = *fixed_pos;
                let offset = u32::unpack(src, fixed_pos)?;
                if *heap_pos as u64 != orig_pos as u64 + offset as u64 {
                    return Err(fracpack::Error::BadOffset);
                }
                Self::unpack(src, heap_pos)
            }
            fn embedded_verify(src: &[u8], fixed_pos: &mut u32, heap_pos: &mut u32) -> fracpack::Result<()> {
                let orig_pos = *fixed_pos;
                let offset = u32::unpack(src, fixed_pos)?;
                if *heap_pos as u64 != orig_pos as u64 + offset as u64 {
                    return Err(fracpack::Error::BadOffset);
                }
                Self::verify(src, heap_pos)
            }
            fn option_fixed_pack(opt: &Option<Self>, dest: &mut Vec<u8>) {
                match opt {
                    Some(x) => x.embedded_fixed_pack(dest),
                    None => dest.extend_from_slice(&1u32.to_le_bytes()),
                }
            }
            fn option_fixed_repack(opt: &Option<Self>, fixed_pos: u32, heap_pos: u32, dest: &mut Vec<u8>) {
                match opt {
                    Some(x) => x.embedded_fixed_repack(fixed_pos, heap_pos, dest),
                    None => (),
                }
            }
            fn option_variable_pack(opt: &Option<Self>, dest: &mut Vec<u8>) {
                match opt {
                    Some(x) => x.embedded_variable_pack(dest),
                    None => (),
                }
            }
            fn option_unpack(src: &[u8], fixed_pos: &mut u32, heap_pos: &mut u32) -> fracpack::Result<Option<Self>> {
                let offset = u32::unpack(src, fixed_pos)?;
                if offset == 1 {
                    return Ok(None);
                }
                *fixed_pos -= 4;
                Ok(Some(Self::embedded_unpack(src, fixed_pos, heap_pos)?))
            }
            fn option_verify(src: &[u8], fixed_pos: &mut u32, heap_pos: &mut u32) -> fracpack::Result<()> {
                let offset = u32::unpack(src, fixed_pos)?;
                if offset == 1 {
                    return Ok(());
                }
                *fixed_pos -= 4;
                Self::embedded_verify(src, fixed_pos, heap_pos)
            }
        }
    })
} // process_enum
