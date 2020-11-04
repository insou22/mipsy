use crate::inst::instruction::ArgumentType;
use serde::{Deserialize, Serialize};

static CONFIG_FILE: &'static str = include_str!("../../mips.yaml");

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct YamlFile {
    pub instructions: Vec<InstructionYaml>,
    pub pseudoinstructions: Vec<PsuedoInstructionYaml>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InstructionYaml {
    pub name: String,
    pub desc_short: Option<String>,
    pub desc_long: Option<String>,
    pub compile: CompileYaml,
    pub runtime: RuntimeYaml,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CompileYaml {
    pub format: Vec<ArgumentType>,
    #[serde(default)]
    pub relative_label: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeYaml {
    #[serde(rename = "type")]
    pub inst_type: InstructionType,
    pub opcode: Option<u8>,
    pub funct: Option<u8>,
    pub rt: Option<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum InstructionType {
    R,
    I,
    J,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PsuedoInstructionYaml {
    pub name: String,
    pub desc_short: Option<String>,
    pub desc_long: Option<String>,
    pub compile: CompileYaml,
    pub expand: Vec<InstructionExpansionYaml>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InstructionExpansionYaml {
    pub inst: String,
    pub data: Vec<String>,
}

// Should be Result -- but not user facing
pub fn get_instructions() -> YamlFile {
    // let file = File::open("mips.yaml").expect("Failed to find mips.yaml!");
    // let yaml: YamlFile = serde_yaml::from_reader(file).expect("Failed to parse mips.yaml!");
    let yaml: YamlFile = serde_yaml::from_str(CONFIG_FILE).expect("Failed to parse mips.yaml!");

    yaml
}
