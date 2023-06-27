//! Home of the `instruction_set!` macro.

use mipsy_instructions::{
    base::{self, InstructionYaml, PseudoInstructionYaml},
    load_instructions, meta,
};
use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use std::{env, fs::File, io::Read, path::PathBuf};

/// # The `instruction_set!` proc macro
///
/// This macro generates a `InstructionSet` struct from a YAML file.
///
/// It does this by parsing the YAML file and generating equivalent Rust code
/// to build the `InstructionSet` struct at compile time.
///
/// This later gets optimised into something more akin to a `static` struct,
/// hopefully of precomputed bytes.
///
/// It is recommended to only invoke this macro in a separate, unique crate,
/// similarly to how mipsy_instructions performs this in the mipsy workspace.
/// This is because the `instruction_set!` macro is a *rather slow* macro to
/// execute, so we want to take advantage of incremental compilation to avoid
/// it being executed every single build.
#[proc_macro]
pub fn instruction_set(input: TokenStream) -> TokenStream {
    let (path, contents) = read_mips_yaml(input);

    let meta_yaml: meta::YamlFile = serde_yaml::from_str(&contents)
        .unwrap_or_else(|_| panic!("Failed to parse {}", path.to_string_lossy()));

    let base_yaml = load_instructions(meta_yaml);

    let mut native_instructions: Vec<proc_macro2::TokenStream> = vec![];
    let mut pseudo_instructions: Vec<proc_macro2::TokenStream> = vec![];

    for instruction in base_yaml.instructions {
        native_instructions.push(quote_instruction(instruction));
    }

    for instruction in base_yaml.pseudoinstructions {
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
    let format = instruction.compile.format.into_iter().map(|arg| {
        use base::ArgumentType;

        let arg_type = match arg {
            ArgumentType::Rd => quote! { Rd },
            ArgumentType::Rs => quote! { Rs },
            ArgumentType::Rt => quote! { Rt },
            ArgumentType::Shamt => quote! { Shamt },
            ArgumentType::I16 => quote! { I16 },
            ArgumentType::U16 => quote! { U16 },
            ArgumentType::J => quote! { J },
            ArgumentType::OffRs => quote! { OffRs },
            ArgumentType::OffRt => quote! { OffRt },
            ArgumentType::F32 => quote! { F32 },
            ArgumentType::F64 => quote! { F64 },
            ArgumentType::I32 => quote! { I32 },
            ArgumentType::U32 => quote! { U32 },
            ArgumentType::Off32Rs => quote! { Off32Rs },
            ArgumentType::Off32Rt => quote! { Off32Rt },
            ArgumentType::Rx => panic!("Rx is not a real register -- it must be macroed away"),
        };

        quote! {
            ::mipsy_lib::inst::ArgumentType::#arg_type
        }
    });
    let relative_label = instruction.compile.relative_label;
    let runtime_signature = {
        use base::InstructionType;

        let inst_type = match instruction.runtime.inst_type {
            InstructionType::R => {
                let opcode = instruction.runtime.opcode.unwrap_or(0);

                let funct = instruction.runtime.funct.unwrap_or(0);

                let shamt = match instruction.runtime.shamt {
                    Some(shamt) => quote! { ::std::option::Option::Some(#shamt) },
                    None => quote! { ::std::option::Option::None },
                };

                let rs = match instruction.runtime.rs {
                    Some(rs) => quote! { ::std::option::Option::Some(#rs) },
                    None => quote! { ::std::option::Option::None },
                };

                let rt = match instruction.runtime.rt {
                    Some(rt) => quote! { ::std::option::Option::Some(#rt) },
                    None => quote! { ::std::option::Option::None },
                };

                let rd = match instruction.runtime.rd {
                    Some(rd) => quote! { ::std::option::Option::Some(#rd) },
                    None => quote! { ::std::option::Option::None },
                };

                quote! { R { opcode: #opcode, funct: #funct, shamt: #shamt, rs: #rs, rt: #rt, rd: #rd } }
            }
            InstructionType::I => {
                let opcode = instruction.runtime.opcode.unwrap_or_else(|| {
                    panic!("invalid mips.yaml: missing opcode for {}", instruction.name)
                });

                let rt = match instruction.runtime.rt {
                    Some(rt) => quote! { ::std::option::Option::Some(#rt) },
                    None => quote! { ::std::option::Option::None },
                };

                quote! { I { opcode: #opcode, rt: #rt } }
            }
            InstructionType::J => {
                let opcode = instruction.runtime.opcode.unwrap_or_else(|| {
                    panic!("invalid mips.yaml: missing opcode for {}", instruction.name)
                });

                quote! { J { opcode: #opcode } }
            }
        };

        quote! {
            ::mipsy_lib::inst::RuntimeSignature::#inst_type
        }
    };

    let reads = instruction.runtime.reads.into_iter().map(|arg| {
        use base::ReadsRegisterType;

        let arg_type = match arg {
            ReadsRegisterType::Rs => quote! { Rs },
            ReadsRegisterType::Rt => quote! { Rt },
            ReadsRegisterType::OffRs => quote! { OffRs },
            ReadsRegisterType::OffRt => quote! { OffRt },
        };

        quote! {
            ::mipsy_lib::inst::ReadsRegisterType::#arg_type
        }
    });

    let desc_short = {
        match instruction.desc_short {
            Some(desc) => {
                quote! { ::std::option::Option::Some(::std::string::String::from(#desc)) }
            }
            None => quote! { ::std::option::Option::None },
        }
    };

    let desc_long = {
        match instruction.desc_long {
            Some(desc) => {
                quote! { ::std::option::Option::Some(::std::string::String::from(#desc)) }
            }
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
            ::mipsy_lib::inst::RuntimeMetadata::new(
                vec![
                    #(#reads),*
                ],
            ),
            ::mipsy_lib::inst::InstMetadata::new(
                #desc_short,
                #desc_long,
            ),
        )
    }
}

fn quote_pseudo_instruction(instruction: PseudoInstructionYaml) -> proc_macro2::TokenStream {
    let name = instruction.name.to_ascii_lowercase();
    let format = instruction.compile.format.into_iter().map(|arg| {
        use base::ArgumentType;

        let arg_type = match arg {
            ArgumentType::Rd => quote! { Rd },
            ArgumentType::Rs => quote! { Rs },
            ArgumentType::Rt => quote! { Rt },
            ArgumentType::Shamt => quote! { Shamt },
            ArgumentType::I16 => quote! { I16 },
            ArgumentType::U16 => quote! { U16 },
            ArgumentType::J => quote! { J },
            ArgumentType::OffRs => quote! { OffRs },
            ArgumentType::OffRt => quote! { OffRt },
            ArgumentType::F32 => quote! { F32 },
            ArgumentType::F64 => quote! { F64 },
            ArgumentType::I32 => quote! { I32 },
            ArgumentType::U32 => quote! { U32 },
            ArgumentType::Off32Rs => quote! { Off32Rs },
            ArgumentType::Off32Rt => quote! { Off32Rt },
            ArgumentType::Rx => panic!("Rx is not a real register -- it must be macroed away"),
        };

        quote! {
            ::mipsy_lib::inst::ArgumentType::#arg_type
        }
    });
    let relative_label = instruction.compile.relative_label;

    let expand = {
        let expansions = instruction.expand.into_iter().map(|expand| {
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
    let project_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");

    match input.into_iter().next() {
        Some(TokenTree::Literal(literal)) => {
            let literal_str = literal.to_string();
            if !literal_str.starts_with('"') || !literal_str.ends_with('"') {
                panic!("Expected a constant string literal path to the mips.yaml template");
            }

            let literal_str = literal_str
                .strip_prefix('"')
                .expect("just checked string starts with single quote")
                .strip_suffix('"')
                .expect("just checked string ends with single quote");

            let mut path = PathBuf::from(project_root);
            path.push(literal_str);

            let mut contents = String::new();

            File::open(&path)
                .unwrap_or_else(|_| panic!("Failed to open file: {}", path.to_string_lossy()))
                .read_to_string(&mut contents)
                .unwrap_or_else(|_| {
                    panic!(
                        "Failed to read as a UTF-8 string: {}",
                        path.to_string_lossy()
                    )
                });

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
