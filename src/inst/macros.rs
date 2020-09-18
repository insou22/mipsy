#[macro_export]
macro_rules! inst_sig {
    ($name:tt, $format:expr) => {
        pub const $name: super::signature::InstructionSignature<'_> = InstructionSignature {
            name: proc_macros::lower_ident_str!($name),
            format: $format,
            pseudo: false,
        };
    };
}

#[macro_export]
macro_rules! pseudo_sig {
    ($name:tt, $format:expr) => {
        pub const $name: super::signature::InstructionSignature<'_> = InstructionSignature {
            name: proc_macros::lower_ident_str!($name),
            format: $format,
            pseudo: true,
        };
    };
}

#[macro_export]
macro_rules! i_format {
    ($name: tt, $format_type: tt) => {
        pub const $name: InstructionFormat = super::signature::InstructionFormat::$format_type;
    };
    ($name: tt, $format_type: tt, $($arg:expr) , +) => {
        pub const $name: InstructionFormat = super::signature::InstructionFormat::$format_type(
            $(
                $arg,
            )+
        );
    };
}