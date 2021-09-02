mod yaml_model;

use std::{env, fs::File, io::Read, path::PathBuf};
use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use yaml_model::{InstructionYaml, PseudoInstructionYaml};

use crate::yaml_model::YamlFile;

#[proc_macro]
pub fn instruction_set(input: TokenStream) -> TokenStream {
    let (path, contents) = read_mips_yaml(input);

    let yaml: YamlFile = serde_yaml::from_str(&contents)
            .expect(&format!("Failed to parse {}", path.to_string_lossy()));
    
    let mut native_instructions: Vec<proc_macro2::TokenStream> = vec![];
    let mut pseudo_instructions: Vec<proc_macro2::TokenStream> = vec![];

    for instruction in yaml.instructions {
        native_instructions.push(quote_instruction(instruction));
    }

    for instruction in yaml.pseudoinstructions {
        pseudo_instructions.push(quote_pseudo_instruction(instruction));      
    }

    let tokens = quote! {
        ::mipsy_lib::inst::InstSet::new(
            vec![#(#native_instructions),*],
            vec![#(#pseudo_instructions),*]
        )
    };
    
    tokens.into()
}

fn quote_instruction(instruction: InstructionYaml) -> proc_macro2::TokenStream {
    let name = instruction.name.to_ascii_lowercase();
    let format = instruction.compile.format
        .into_iter()
        .map(|arg| {
            use yaml_model::ArgumentType;

            let arg_type = match arg {
                ArgumentType::Rd      => quote! { Rd },
                ArgumentType::Rs      => quote! { Rs },
                ArgumentType::Rt      => quote! { Rt },
                ArgumentType::Shamt   => quote! { Shamt },
                ArgumentType::I16     => quote! { I16 },
                ArgumentType::U16     => quote! { U16 },
                ArgumentType::J       => quote! { J },
                ArgumentType::OffRs   => quote! { OffRs },
                ArgumentType::OffRt   => quote! { OffRt },
                ArgumentType::F32     => quote! { F32 },
                ArgumentType::F64     => quote! { F64 },
                ArgumentType::I32     => quote! { I32 },
                ArgumentType::U32     => quote! { U32 },
                ArgumentType::Off32Rs => quote! { Off32Rs },
                ArgumentType::Off32Rt => quote! { Off32Rt },
            };

            quote! {
                ::mipsy_lib::inst::ArgumentType::#arg_type
            }
        });
    let relative_label = instruction.compile.relative_label;
    let runtime_signature = {
        use yaml_model::InstructionType;

        let inst_type = match instruction.runtime.inst_type {
            InstructionType::R => {
                let funct = instruction.runtime.funct
                        .expect(&format!("invalid mips.yaml: missing funct for {}", instruction.name));

                quote! { R { funct: #funct } }
            }
            InstructionType::I => {
                let opcode = instruction.runtime.opcode
                        .expect(&format!("invalid mips.yaml: missing opcode for {}", instruction.name));

                let rt = match instruction.runtime.rt {
                    Some(rt) => quote! { ::std::option::Option::Some(#rt) },
                    None     => quote! { ::std::option::Option::None },
                };

                quote! { I { opcode: #opcode, rt: #rt } }
            }
            InstructionType::J => {
                let opcode = instruction.runtime.opcode
                        .expect(&format!("invalid mips.yaml: missing opcode for {}", instruction.name));

                quote! { J { opcode: #opcode } }
            }
        };

        quote! {
            ::mipsy_lib::inst::RuntimeSignature::#inst_type
        }
    };

    let desc_short = {
        match instruction.desc_short {
            Some(desc) => quote! { ::std::option::Option::Some(::std::string::String::from(#desc)) },
            None => quote! { ::std::option::Option::None },
        }
    };

    let desc_long = {
        match instruction.desc_long {
            Some(desc) => quote! { ::std::option::Option::Some(::std::string::String::from(#desc)) },
            None => quote! { ::std::option::Option::None },
        }
    };
    
    quote! {
        ::mipsy_lib::inst::InstSignature::new(
            ::std::string::String::from(#name),
            ::mipsy_lib::inst::CompileSignature::new(
                vec![
                    #(#format),*
                ],
                #relative_label,
            ),
            #runtime_signature,
            ::mipsy_lib::inst::InstMetadata::new(
                #desc_short,
                #desc_long,
            ),
        )
    }
}

fn quote_pseudo_instruction(instruction: PseudoInstructionYaml) -> proc_macro2::TokenStream {
    let name = instruction.name.to_ascii_lowercase();
    let format = instruction.compile.format
        .into_iter()
        .map(|arg| {
            use yaml_model::ArgumentType;

            let arg_type = match arg {
                ArgumentType::Rd      => quote! { Rd },
                ArgumentType::Rs      => quote! { Rs },
                ArgumentType::Rt      => quote! { Rt },
                ArgumentType::Shamt   => quote! { Shamt },
                ArgumentType::I16     => quote! { I16 },
                ArgumentType::U16     => quote! { U16 },
                ArgumentType::J       => quote! { J },
                ArgumentType::OffRs   => quote! { OffRs },
                ArgumentType::OffRt   => quote! { OffRt },
                ArgumentType::F32     => quote! { F32 },
                ArgumentType::F64     => quote! { F64 },
                ArgumentType::I32     => quote! { I32 },
                ArgumentType::U32     => quote! { U32 },
                ArgumentType::Off32Rs => quote! { Off32Rs },
                ArgumentType::Off32Rt => quote! { Off32Rt },
            };

            quote! {
                ::mipsy_lib::inst::ArgumentType::#arg_type
            }
        });
    let relative_label = instruction.compile.relative_label;

    let expand = {
        let expansions = instruction.expand.into_iter()
            .map(|expand| {
                let inst = expand.inst;
                let data = expand.data;

                quote! {
                    ::mipsy_lib::inst::PseudoExpand::new(
                        ::std::string::String::from(#inst),
                        vec![
                            #(::std::string::String::from(#data)),*
                        ]
                    )
                }
            });
        
        quote! {
            vec![
                #(#expansions),*
            ]
        }
    };
    
    quote! {
        ::mipsy_lib::inst::PseudoSignature::new(
            ::std::string::String::from(#name),
            ::mipsy_lib::inst::CompileSignature::new(
                vec![
                    #(#format),*
                ],
                #relative_label,
            ),
            #expand,
        )
    }
}

fn read_mips_yaml(input: TokenStream) -> (PathBuf, String) {
    let project_root = env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR is not set");

    match input.into_iter().next() {
        Some(TokenTree::Literal(literal)) => {
            let literal_str = literal.to_string();
            if !literal_str.starts_with('"') || !literal_str.ends_with('"') {
                panic!("Expected a constant string literal path to the mips.yaml template");
            }

            let literal_str = literal_str.strip_prefix('"')
                    .expect("just checked string starts with single quote")
                    .strip_suffix('"')
                    .expect("just checked string ends with single quote");

            let mut path = PathBuf::from(project_root);
            path.push(literal_str);

            let mut contents = String::new();

            File::open(&path)
                    .expect(&format!("Failed to open file: {}", path.to_string_lossy()))
                    .read_to_string(&mut contents)
                    .expect(&format!("Failed to read as a UTF-8 string: {}", path.to_string_lossy()));

            (path, contents)           
        }
        Some(_) => {
            panic!("Expected a constant string literal path to the mips.yaml template");
        }
        None => {
            panic!("Expected a constant string literal path to the mips.yaml template");
        }
    }
}