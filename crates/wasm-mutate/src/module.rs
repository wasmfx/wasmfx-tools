use crate::Result;
use wasm_encoder::{BlockType, HeapType, RefType, ValType};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PrimitiveTypeInfo {
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
    Empty,
}

#[derive(Debug, Clone)]
pub struct FuncInfo {
    pub params: Vec<PrimitiveTypeInfo>,
    pub returns: Vec<PrimitiveTypeInfo>,
}

#[derive(Debug, Clone)]
pub enum TypeInfo {
    Func(FuncInfo),
    // TODO: module linking support will require instance and module types.
}

impl From<wasmparser::ValType> for PrimitiveTypeInfo {
    fn from(value: wasmparser::ValType) -> Self {
        match value {
            wasmparser::ValType::I32 => PrimitiveTypeInfo::I32,
            wasmparser::ValType::I64 => PrimitiveTypeInfo::I64,
            wasmparser::ValType::F32 => PrimitiveTypeInfo::F32,
            wasmparser::ValType::F64 => PrimitiveTypeInfo::F64,
            wasmparser::ValType::V128 => PrimitiveTypeInfo::V128,
            wasmparser::ValType::Ref(t) => t.into(),
        }
    }
}

impl From<wasmparser::RefType> for PrimitiveTypeInfo {
    fn from(value: wasmparser::RefType) -> Self {
        match value {
            wasmparser::RefType::FUNCREF => PrimitiveTypeInfo::FuncRef,
            wasmparser::RefType::EXTERNREF => PrimitiveTypeInfo::ExternRef,
            _ => unimplemented!(),
        }
    }
}

impl From<wasmparser::FuncType> for TypeInfo {
    fn from(ft: wasmparser::FuncType) -> Self {
        TypeInfo::Func(FuncInfo {
            params: ft
                .params()
                .iter()
                .map(|&t| PrimitiveTypeInfo::from(t))
                .collect(),
            returns: ft
                .results()
                .iter()
                .map(|&t| PrimitiveTypeInfo::from(t))
                .collect(),
        })
    }
}

// TODO(dhil): Remove this when `into_iter_err_on_gc_types` has been
// redesigned.
impl From<wasmparser::FuncOrContType> for TypeInfo {
    fn from(t: wasmparser::FuncOrContType) -> Self {
        match t {
            wasmparser::FuncOrContType::Func(f) => f.into(),
            wasmparser::FuncOrContType::Cont(_) => todo!("continuation types are not supported"),
        }
    }
}

pub fn map_type(tpe: wasmparser::ValType) -> Result<ValType> {
    match tpe {
        wasmparser::ValType::I32 => Ok(ValType::I32),
        wasmparser::ValType::I64 => Ok(ValType::I64),
        wasmparser::ValType::F32 => Ok(ValType::F32),
        wasmparser::ValType::F64 => Ok(ValType::F64),
        wasmparser::ValType::V128 => Ok(ValType::V128),
        wasmparser::ValType::Ref(t) => Ok(ValType::Ref(map_ref_type(t)?)),
    }
}

pub fn map_ref_type(ref_ty: wasmparser::RefType) -> Result<RefType> {
    Ok(RefType {
        nullable: ref_ty.is_nullable(),
        heap_type: match ref_ty.heap_type() {
            wasmparser::HeapType::Concrete(i) => HeapType::Concrete(i.as_module_index().unwrap()),
            wasmparser::HeapType::Abstract { shared, ty } => {
                let ty = ty.into();
                HeapType::Abstract { shared, ty }
            }
        },
    })
}

pub fn map_block_type(ty: wasmparser::BlockType) -> Result<BlockType> {
    match ty {
        wasmparser::BlockType::Empty => Ok(BlockType::Empty),
        wasmparser::BlockType::Type(ty) => Ok(BlockType::Result(map_type(ty)?)),
        wasmparser::BlockType::FuncType(f) => Ok(BlockType::FunctionType(f)),
    }
}

// The SectionId is stored as a `u8`. This macro will ensure that all of the `SectionId`s are
// matched and also takes care of converting the patterns to `u8` for matching.
macro_rules! match_section_id {
    (match $scrutinee:expr;
        $($pat:ident => $result:expr,)*
        _ => $otherwise:expr,
    ) => {'result: loop {
        #[allow(unreachable_code, non_upper_case_globals)]
        {
            $(const $pat: u8 = SectionId::$pat as u8;)*
            break 'result match $scrutinee {
                $($pat => $result,)*
                _ => $otherwise,
            };
            // Check exhaustiveness of the SectionId match
            match SectionId::Type {
                $(SectionId::$pat => (),)*
            };
        }
    }}
}
pub(crate) use match_section_id;
