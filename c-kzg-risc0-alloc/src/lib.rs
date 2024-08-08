// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse, parse2};

#[proc_macro_attribute]
pub fn risc0_c_kzg_alloc_mod(_attr: TokenStream, input: TokenStream) -> TokenStream {
    // let input: syn::Item = parse(input).unwrap();
    let mut module_def: syn::ItemMod = parse(input).expect("Failed to parse input as a module");
    let builtin_module_tokens = quote! {
        mod __risc0_c_kzg_alloc_mod {
            extern crate alloc;

            const LEN_SIZE: usize = core::mem::size_of::<usize>();

            unsafe fn store_len(block_ptr: *mut u8, size: usize) -> *mut u8 {
                let len_ptr = block_ptr as *mut usize;
                *len_ptr = size;

                block_ptr.offset(LEN_SIZE as isize)
            }

            unsafe fn get_len(block_ptr: *mut u8) -> (usize, *mut u8) {
                let len_ptr = block_ptr.offset(-(LEN_SIZE as isize)) as *mut usize;
                (*len_ptr, len_ptr as *mut u8)
            }

            #[no_mangle]
            unsafe extern "C" fn malloc(size: usize) -> *mut core::ffi::c_void {
                let layout = core::alloc::Layout::from_size_align(size + LEN_SIZE, 4)
                    .expect("unable to construct memory layout");
                let block_ptr = alloc::alloc::alloc(layout);

                if block_ptr.is_null() {
                    alloc::alloc::handle_alloc_error(layout);
                }

                let data_ptr = store_len(block_ptr, size);

                data_ptr as *mut core::ffi::c_void
            }

            #[no_mangle]
            unsafe extern "C" fn calloc(nobj: usize, size: usize) -> *mut core::ffi::c_void {
                let size = nobj * size;
                let layout = core::alloc::Layout::from_size_align(size + LEN_SIZE, 4)
                    .expect("unable to construct memory layout");
                let block_ptr = alloc::alloc::alloc_zeroed(layout);

                if block_ptr.is_null() {
                    alloc::alloc::handle_alloc_error(layout);
                }

                let data_ptr = store_len(block_ptr, size);

                data_ptr as *mut core::ffi::c_void
            }

            #[no_mangle]
            unsafe extern "C" fn free(block_ptr: *const core::ffi::c_void) {
                let (size, free_ptr) = get_len(block_ptr as *mut u8);

                let layout = core::alloc::Layout::from_size_align(size, 4)
                    .expect("unable to construct memory layout");

                alloc::alloc::dealloc(free_ptr, layout);
            }
        }
    };
    let builtin_module_parsed: syn::ItemMod =
        parse2(builtin_module_tokens).expect("Failed to parse built in definitions as a module");
    if let Some(contents) = module_def.content.as_mut() {
        let new_items = builtin_module_parsed.content.unwrap().1;
        contents.1.extend(new_items.into_iter());
    } else {
        module_def.content = builtin_module_parsed.content;
    }
    module_def.into_token_stream().into()
}
