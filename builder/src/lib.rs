use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, AngleBracketedGenericArguments, Data, DeriveInput, Fields, Ident, Path,
    PathArguments, Type, TypePath,
};

#[rustfmt::skip::macros(quote)]
#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let ident = derive_input.ident;
    let builder_ident = Ident::new(&format!("{}Builder", ident), ident.span());

    let mut struct_fields = vec![];
    let mut struct_types = vec![];
    let mut optional_fields = vec![];
    let mut optional_types = vec![];

    if let Data::Struct(data) = derive_input.data {
        if let Fields::Named(fields) = data.fields {
            for field in fields.named.into_iter() {
                match field.ty {
                    Type::Path(TypePath {
                        qself: _,
                        path:
                            Path {
                                leading_colon: _,
                                segments: ref path_segments,
                            },
                    }) => {
                        if path_segments[0].ident == "Option" {
                            if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                                colon2_token: _,
                                lt_token: _,
                                gt_token: _,
                                args: under_option_args,
                            }) = &path_segments[0].arguments
                            {
                                optional_types.push(under_option_args[0].clone());
                                optional_fields.push(field.ident.unwrap());
                            } else {
                                panic!("lmao");
                            }
                        } else {
                            struct_fields.push(field.ident.unwrap());
                            struct_types.push(field.ty);
                        }
                    }
                    _ => panic!("oops"),
                }
            }
        }
    }
    let tokens = quote! {
    use std::error::Error;

    pub struct #builder_ident {
        #(#struct_fields: Option<#struct_types>),*,
	#(#optional_fields: Option<#optional_types>),*
    }

    impl #builder_ident {
        #(fn #struct_fields(&mut self, #struct_fields: #struct_types) -> &mut Self {
	    self.#struct_fields = Some(#struct_fields);
	    self
        })*
	
	#(fn #optional_fields(&mut self, #optional_fields: #optional_types) -> &mut Self {
	    self.#optional_fields = Some(#optional_fields);
	    self
        })*

        pub fn build(&mut self) -> Result<#ident, Box<dyn Error>> {
	    Ok(#ident {
		#(#struct_fields: self.#struct_fields.take().ok_or("sosi")?),*,
		#(#optional_fields: self.#optional_fields.take()),*
	    })
        }
    }

    impl #ident {
        pub fn builder() -> #builder_ident {
	    #builder_ident {
		#(#struct_fields: None),*,
		#(#optional_fields: None),*
	    }
        }
    }
    };
    tokens.into()
}
