//! # mipsy_instructions
//!
//! This crate, on it's surface, seems incredibly useless.
//! It only exports a single function, `inst_set`, which
//! only calls a single macro, the `instruction_set!` macro.
//!
//! The `instruction_set!` macro is used to parse the MIPS
//! instruction set and generate a Rust struct that can be
//! used to decode the instructions.
//!
//! This macro, however, is *rather slow* and usually won't
//! need to be recompiled often. On my Ryzen 7 5800X, it
//! takes ~25 seconds to run the macro, and this computation
//! doesn't seem possible to parallelize as rustc seems to
//! only allocate a single thread for a proc-macro computaion.
//!
//! By putting this macro invocation in a separate crate,
//! incremental compilation allows us to only compile this crate
//! once, and then only recompile it if the instruction set
//! changes (although this would have to be a manual `clean`).

pub mod base;
pub mod meta;

use crate::meta::DeriveStatementYaml;
#[allow(unused_imports)] // rust-analyzer seems to think this is unused, but it's not
use mipsy_lib::InstSet;

#[cfg(feature = "rt_yaml")]
static MIPS_YAML: &str = include_str!("../../../mips.yaml");

#[cfg(feature = "rt_yaml")]
pub fn inst_set() -> InstSet {
    let meta_yaml: meta::YamlFile =
        serde_yaml::from_str(MIPS_YAML).unwrap_or_else(|_| panic!("Failed to parse mips.yaml"));

    let base_yaml = load_instructions(meta_yaml);

    let inst_set_native = base_yaml.instructions.into_iter().map(Into::into).collect();
    let inst_set_pseudo = base_yaml
        .pseudoinstructions
        .into_iter()
        .map(Into::into)
        .collect();

    InstSet::new(inst_set_native, inst_set_pseudo)
}

pub fn load_instructions(meta_yaml: meta::YamlFile) -> base::YamlFile {
    let mut base_yaml = base::YamlFile {
        instructions: vec![],
        pseudoinstructions: vec![],
    };

    for instruction in meta_yaml.instructions {
        base_yaml.instructions.push(base::InstructionYaml {
            name: instruction.name,
            desc_short: instruction.desc_short,
            desc_long: instruction.desc_long,
            compile: base::CompileYaml {
                format: instruction
                    .compile
                    .format
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                relative_label: instruction.compile.relative_label,
            },
            runtime: base::RuntimeYaml {
                inst_type: instruction.runtime.inst_type.into(),
                opcode: instruction.runtime.opcode,
                funct: instruction.runtime.funct,
                shamt: instruction.runtime.shamt,
                rt: instruction.runtime.rt,
                rs: instruction.runtime.rs,
                rd: instruction.runtime.rd,
                reads: instruction
                    .runtime
                    .reads
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            },
        });
    }

    for instruction in meta_yaml.pseudoinstructions {
        let base = base::PseudoInstructionYaml {
            name: instruction.name.clone(),
            desc_short: instruction.desc_short.clone(),
            desc_long: instruction.desc_long.clone(),
            compile: base::CompileYaml {
                format: instruction
                    .compile
                    .format
                    .iter()
                    .cloned()
                    .map(Into::into)
                    .collect(),
                relative_label: instruction.compile.relative_label,
            },
            expand: instruction.expand.iter().cloned().map(Into::into).collect(),
        };

        let only_derive = instruction.only_derive;

        let derives = expand_all_derives(instruction);

        // if !derives.is_empty() {
        //     println!("{:?}", derives);
        // }

        if !only_derive {
            base_yaml.pseudoinstructions.push(base);
        }
        base_yaml.pseudoinstructions.extend(derives);
    }

    base_yaml
}

fn expand_all_derives(of: meta::PseudoInstructionYaml) -> Vec<base::PseudoInstructionYaml> {
    let mut all_derives = vec![];

    for derive in of.derives.iter() {
        let expandeds = expand_one_derive(&of, derive);

        for expanded in expandeds {
            all_derives.push(base::PseudoInstructionYaml {
                name: expanded.name.clone(),
                desc_short: expanded.desc_short.clone(),
                desc_long: expanded.desc_long.clone(),
                compile: base::CompileYaml {
                    format: expanded
                        .compile
                        .format
                        .iter()
                        .cloned()
                        .map(Into::into)
                        .collect(),
                    relative_label: expanded.compile.relative_label,
                },
                expand: expanded.expand.iter().cloned().map(Into::into).collect(),
            });

            all_derives.extend(expand_all_derives(expanded));
        }
    }

    all_derives
}

fn expand_one_derive(
    of: &meta::PseudoInstructionYaml,
    derive: &DeriveStatementYaml,
) -> Vec<meta::PseudoInstructionYaml> {
    match derive {
        DeriveStatementYaml::Imm2Reg {
            register,
            imm_types,
            sign_extend,
            imm_register,
            derives,
        } => imm_types
            .iter()
            .map(|imm_type| {
                use meta::Imm2RegImmType::*;
                let imm_register = format!("${}", imm_register.as_deref().unwrap_or("At"));

                let mut expansion = match imm_type {
                    I16 => vec![meta::InstructionExpansionYaml {
                        inst: if *sign_extend { "ADDI" } else { "ORI" }.to_string(),
                        data: vec![
                            imm_register.to_string(),
                            "$0".to_string(),
                            "$I16".to_string(),
                        ],
                    }],
                    U16 => vec![meta::InstructionExpansionYaml {
                        inst: "ORI".to_string(),
                        data: vec![
                            imm_register.to_string(),
                            "$0".to_string(),
                            "$U16".to_string(),
                        ],
                    }],
                    I32 => vec![
                        meta::InstructionExpansionYaml {
                            inst: "LUI".to_string(),
                            data: vec![imm_register.to_string(), "$I32uHi".to_string()],
                        },
                        meta::InstructionExpansionYaml {
                            inst: "ORI".to_string(),
                            data: vec![
                                imm_register.to_string(),
                                imm_register.to_string(),
                                "$I32uLo".to_string(),
                            ],
                        },
                    ],
                    U32 => vec![
                        meta::InstructionExpansionYaml {
                            inst: "LUI".to_string(),
                            data: vec![imm_register.to_string(), "$U32uHi".to_string()],
                        },
                        meta::InstructionExpansionYaml {
                            inst: "ORI".to_string(),
                            data: vec![
                                imm_register.to_string(),
                                imm_register.to_string(),
                                "$U32uLo".to_string(),
                            ],
                        },
                    ],
                };

                expansion.extend(of.expand.iter().map(|expand| {
                    meta::InstructionExpansionYaml {
                        inst: expand.inst.clone(),
                        data: expand
                            .data
                            .iter()
                            .map(|data| {
                                if data == &format!("${}", register) {
                                    imm_register.to_string()
                                } else {
                                    data.to_string()
                                }
                            })
                            .collect(),
                    }
                }));

                let arg_type = match imm_type {
                    I16 => meta::ArgumentType::I16,
                    U16 => meta::ArgumentType::U16,
                    I32 => meta::ArgumentType::I32,
                    U32 => meta::ArgumentType::U32,
                };

                meta::PseudoInstructionYaml {
                    name: of.name.clone(),
                    desc_short: of.desc_short.clone(),
                    desc_long: of.desc_long.clone(),
                    compile: meta::CompileYaml {
                        format: of
                            .compile
                            .format
                            .iter()
                            .map(|arg| {
                                if &arg.to_string() == register {
                                    arg_type
                                } else {
                                    *arg
                                }
                            })
                            .collect(),
                        relative_label: of.compile.relative_label,
                    },
                    expand: expansion,
                    only_derive: false,
                    derives: derives.clone(),
                }
            })
            .collect(),
        DeriveStatementYaml::DefaultValue {
            value,
            default,
            derives,
        } => {
            vec![meta::PseudoInstructionYaml {
                name: of.name.clone(),
                desc_short: of.desc_short.clone(),
                desc_long: of.desc_long.clone(),
                compile: meta::CompileYaml {
                    format: of
                        .compile
                        .format
                        .iter()
                        .filter(|arg| arg != &value)
                        .cloned()
                        .collect(),
                    relative_label: of.compile.relative_label,
                },
                expand: of
                    .expand
                    .iter()
                    .map(|expand| meta::InstructionExpansionYaml {
                        inst: expand.inst.clone(),
                        data: expand
                            .data
                            .iter()
                            // TODO(zkol): this format is incredibly stupid
                            .map(|arg| arg.replace(&format!("${}", value), default))
                            .collect(),
                    })
                    .collect(),
                only_derive: false,
                derives: derives.clone(),
            }]
        }
    }
}
