use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IRFunTy {
    pub param_tys: Vec<IRTy>,
    pub ret_ty: Box<IRTy>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum IRTy {
    UnitTy,    // ts: void
    BooleanTy, // ts: boolean
    Float64Ty, // ts: number
    BigIntTy,  // ts: bigint
    StringTy,  // ts: string
    FunTy(Box<IRFunTy>),

    UnknownTy, // no type declared
}

pub trait TypeOf {
    fn type_of(&self) -> IRTy;
}
