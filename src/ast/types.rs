use std::fmt::{Display, Formatter};

use crate::string_pool::PoolS;

#[derive(Clone)]
#[derive(Hash, Eq, PartialEq)]
pub enum TypeName {
    Primitive(PoolS),
    Optional(Box<TypeName>),
    Deref(Box<TypeName>),
    Parameterized(PoolS, Vec<TypeName>),

    Unset,
    Error,

    //Int32,
    //Float,

    UnspecificNumeric
}

impl TypeName {
    pub fn to_specific(self) -> TypeName {
        match self {
            TypeName::Primitive(what) => {
                if what.eq_utf8("int") {
                    return TypeName::Int32;
                }
                if what.eq_utf8("float") {
                    return TypeName::Float;
                }
                self
            }
            _ => {
                self
            }
        }
    }

    pub fn is_specific_numeric(&self) -> bool {
        match self {
            TypeName::Float => true,
            TypeName::Int32 => true,
            _ => false
        }
    }

    pub fn eq_or_may_coerce(&self, rhs: &TypeName) -> bool {
        // IF the LHS is a specific number and the RHS is an unspecific number, it is possible that
        // the LHS can propagate its type to the RHS.
        //
        // There used to be a bug, where we said if the RHS is also a specific numeric, it can
        // coerce... this is NOT TRUE! The only number types allowed to be automatically coerced
        // are UnspecificNumeric (and, if we add an UnspecificFloat at some point, that one).
        if self.is_specific_numeric() && rhs == &TypeName::UnspecificNumeric {
            return true;
        }
        return self == rhs;
    }
}

impl Display for TypeName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            TypeName::Primitive(what) => {
                f.write_fmt(format_args!("T{}", what))?;
            }
            TypeName::Optional(inner) => {
                f.write_fmt(format_args!("Op{}", inner.as_ref()))?;
            }
            TypeName::Deref(inner) => {
                f.write_fmt(format_args!("Dr{}", inner.as_ref()))?;
            }
            TypeName::Parameterized(id, others) => {
                f.write_fmt(format_args!("Par{}", id))?;
                f.write_str("W")?;
                for other in others {
                    other.fmt(f)?;
                }
            }
            TypeName::Void => {
                f.write_str("void")?;
            }
            TypeName::Unset => {
                f.write_str("INFER_ERR")?;
            }
            TypeName::Error => {
                f.write_str("BadType")?;
            }
            TypeName::Int32 => {
                f.write_str("int32_t")?;
            }
            TypeName::Float => {
                f.write_str("float")?;
            }
            TypeName::UnspecificNumeric => {
                f.write_str("NUMERIC_ERR")?;
            }
        }

        Ok(())
    }
}