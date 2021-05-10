use crate::yaml::{
    YamlFile,
    InstructionType,
};
use super::instruction::{
    InstSet,
    InstSignature,
    CompileSignature,
    RuntimeSignature,
    InstMetadata,
    PseudoSignature,
    PseudoExpand,
};

pub fn from_yaml(yaml: &YamlFile) -> InstSet {
    let mut native_set = vec![];

    for inst in &yaml.instructions {
        let native_inst = InstSignature {
            name: inst.name.to_ascii_lowercase(),
            compile: CompileSignature {
                format: inst.compile.format.clone(),
                relative_label: inst.compile.relative_label,
            },
            runtime: match inst.runtime.inst_type {
                InstructionType::R => {
                    if let Some(funct) = inst.runtime.funct {
                        RuntimeSignature::R { funct }
                    } else {
                        panic!("invalid mips.yaml: missing funct for {}", inst.name);
                    }
                }
                InstructionType::I => {
                    if let Some(opcode) = inst.runtime.opcode {
                        RuntimeSignature::I { opcode, rt: inst.runtime.rt }
                    } else {
                        panic!("invalid mips.yaml: missing opcode for {}", inst.name);
                    }
                }
                InstructionType::J => {
                    if let Some(opcode) = inst.runtime.opcode {
                        RuntimeSignature::J { opcode }
                    } else {
                        panic!("invalid mips.yaml: missing opcode for {}", inst.name);
                    }
                }
            },
            meta: InstMetadata {
                desc_short: inst.desc_short.clone(),
                desc_long: inst.desc_long.clone(),
            },
        };

        native_set.push(native_inst);
    }

    let mut pseudo_set = vec![];

    for inst in &yaml.pseudoinstructions {
        let pseudo_inst = PseudoSignature {
            name: inst.name.to_ascii_lowercase(),
            compile: CompileSignature {
                format: inst.compile.format.clone(),
                relative_label: inst.compile.relative_label,
            },
            expand: inst.expand.iter()
                    .map(|expand| PseudoExpand {
                        inst: expand.inst.clone(),
                        data: expand.data.clone(),
                    }).collect(),
        };

        pseudo_set.push(pseudo_inst);
    }

    InstSet {
        native_set,
        pseudo_set,
    }
}
