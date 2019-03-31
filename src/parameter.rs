use optee_teec_sys as raw;
use std::mem;

/// Parameters is a tuple of four Parameters.
pub struct Parameters(pub Parameter, pub Parameter, pub Parameter, pub Parameter);

impl Parameters {
    pub fn new(teec_params: [raw::TEEC_Parameter; 4], param_types: u32) -> Self {
        let (f0, f1, f2, f3) = ParamTypes::from(param_types).into_flags();
        let p0 = Parameter::from_raw(teec_params[0], f0);
        let p1 = Parameter::from_raw(teec_params[1], f1);
        let p2 = Parameter::from_raw(teec_params[2], f2);
        let p3 = Parameter::from_raw(teec_params[3], f3);

        Parameters(p0, p1, p2, p3)
    }

    pub fn first(&self) -> &Parameter {
        &self.0
    }

    pub fn second(&self) -> &Parameter {
        &self.1
    }

    pub fn third(&self) -> &Parameter {
        &self.2
    }

    pub fn fourth(&self) -> &Parameter {
        &self.3
    }
}

/// This type defines a Parameter of a Operation. It can be a Temporary Memory
/// Reference, a Registered Memory Reference, or a Value Parameter.
pub struct Parameter {
    raw: raw::TEEC_Parameter,
    pub param_type: ParamType,
}

impl Parameter {
    pub fn new() -> Self {
        let raw = unsafe { mem::zeroed() };
        Self {
            raw: raw,
            param_type: ParamType::None,
        }
    }

    pub fn from_value(a: u32, b: u32, param_type: ParamType) -> Self {
        let raw = raw::TEEC_Parameter {
            value: raw::TEEC_Value { a, b },
        };
        Self {
            raw: raw,
            param_type: param_type,
        }
    }

    pub fn from_tmpref<T>(buffer: *mut T, size: usize, param_type: ParamType) -> Self {
        let raw = raw::TEEC_Parameter {
            tmpref: raw::TEEC_TempMemoryReference {
                buffer: buffer as *mut T as _,
                size: size as libc::size_t,
            },
        };
        Self {
            raw: raw,
            param_type: param_type,
        }
    }

    pub fn from_raw(raw: raw::TEEC_Parameter, param_type: ParamType) -> Self {
        Self {
            raw: raw,
            param_type: param_type,
        }
    }

    pub fn value(&self) -> (u32, u32) {
        unsafe { (self.raw.value.a, self.raw.value.b) }
    }

    pub fn set_value(&mut self, a: u32, b: u32) {
        unsafe {
            self.raw.value.a = a;
            self.raw.value.b = b;
        }
    }

    pub fn tmpref<T>(&mut self) -> *mut T {
        unsafe {
            self.raw.tmpref.buffer as *mut T
        }
    }

    pub fn set_param_type(&mut self, param_type: ParamType) {
        self.param_type = param_type;
    }
}

impl From<Parameter> for raw::TEEC_Parameter {
    fn from(a: Parameter) -> raw::TEEC_Parameter {
        a.raw
    }
}

/// These are used to indicate the type of Parameter encoded inside the
/// operation structure.
#[derive(Copy, Clone)]
pub enum ParamType {
    /// The Parameter is not used.
    None = 0,
    /// The Parameter is a TEEC_Value tagged as input.
    ValueInput = 1,
    /// The Parameter is a TEEC_Value tagged as output.
    ValueOutput = 2,
    /// The Parameter is a TEEC_Value tagged as both as input and output, i.e.,
    /// for which both the behaviors of ValueInput and ValueOutput apply.
    ValueInout = 3,
    /// The Parameter is a TEEC_TempMemoryReference describing a region of
    /// memory which needs to be temporarily registered for the duration of the
    /// Operation and is tagged as input.
    MemrefTempInput = 5,
    /// Same as MemrefTempInput, but the Memory Reference is tagged as
    /// output. The Implementation may update the size field to reflect the
    /// required output size in some use cases.
    MemrefTempOutput = 6,
    /// A Temporary Memory Reference tagged as both input and output, i.e., for
    /// which both the behaviors of MemrefTempInput and MemrefTempOutput apply.
    MemrefTempInout = 7,
    /// The Parameter is a Registered Memory Reference that refers to the
    /// entirety of its parent Shared Memory block. The parameter structure is a
    /// TEEC_MemoryReference. In this structure, the Implementation MUST read
    /// only the parent field and MAY update the size field when the operation
    /// completes.
    MemrefWhole = 0xC,
    /// A Registered Memory Reference structure that refers to a partial region
    /// of its parent Shared Memory block and is tagged as input.
    MemrefPartialInput = 0xD,
    /// A Registered Memory Reference structure that refers to a partial region
    /// of its parent Shared Memory block and is tagged as output.
    MemrefPartialOutput = 0xE,
    /// The Registered Memory Reference structure that refers to a partial
    /// region of its parent Shared Memory block and is tagged as both input and
    /// output, i.e., for which both the behaviors of MemrefPartialInput and
    /// MemrefPartialOutput apply.
    MemrefPartialInout = 0xF,
}

impl From<u32> for ParamType {
    fn from(value: u32) -> Self {
        match value {
            0 => ParamType::None,
            1 => ParamType::ValueInput,
            2 => ParamType::ValueOutput,
            3 => ParamType::ValueInout,
            5 => ParamType::MemrefTempInput,
            6 => ParamType::MemrefTempOutput,
            7 => ParamType::MemrefTempInout,
            0xC => ParamType::MemrefWhole,
            0xD => ParamType::MemrefPartialInput,
            0xE => ParamType::MemrefPartialOutput,
            0xF => ParamType::MemrefPartialInout,
            _ => ParamType::None,
        }
    }
}

pub struct ParamTypes(u32);

impl ParamTypes {
    pub fn new(p0: ParamType, p1: ParamType, p2: ParamType, p3: ParamType) -> Self {
        ParamTypes((p0 as u32) | (p1 as u32) << 4 | (p2 as u32) << 8 | (p3 as u32) << 12)
    }

    pub fn into_flags(&self) -> (ParamType, ParamType, ParamType, ParamType) {
        (
            (0x000fu32 & self.0).into(),
            (0x00f0u32 & self.0).into(),
            (0x0f00u32 & self.0).into(),
            (0xf000u32 & self.0).into(),
        )
    }
}

impl From<u32> for ParamTypes {
    fn from(value: u32) -> Self {
        ParamTypes(value)
    }
}

impl From<[u32; 4]> for ParamTypes {
    fn from(param_types: [u32; 4]) -> Self {
        ParamTypes(
            param_types[0] | param_types[1] << 4 | param_types[2] << 8 | param_types[3] << 12,
        )
    }
}

impl From<ParamTypes> for u32 {
    fn from(a: ParamTypes) -> u32 {
        a.0
    }
}
