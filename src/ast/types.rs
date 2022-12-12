use std::fmt::{Display, Formatter};

use std::io::Write;
use crate::ast::Type::UnspecificNumeric;
use crate::string_pool::PoolS;

#[derive(Clone)]
#[derive(PartialEq)]
pub enum Type {
    Primitive(PoolS),
    Optional(Box<Type>),
    Deref(Box<Type>),
    Parameterized(PoolS, Vec<Type>),
    Void,
    Unset,
    Error,

    Int32,
    Float,

    UnspecificNumeric
}

impl Type {
    pub fn to_specific(self) -> Type {
        match self {
            Type::Primitive(what) => {
                if what.eq_utf8("int") {
                    return Type::Int32;
                }
                if what.eq_utf8("float") {
                    return Type::Float;
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
            Type::Float => true,
            Type::Int32 => true,
            _ => false
        }
    }

    pub fn eq_or_may_coerce(&self, rhs: &Type) -> bool {
        if self.is_specific_numeric() {
            return rhs.is_specific_numeric() || rhs == &UnspecificNumeric;
        }
        return self == rhs;
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Type::Primitive(what) => {
                f.write_fmt(format_args!("T{}", what))?;
            }
            Type::Optional(inner) => {
                f.write_fmt(format_args!("Op{}", inner.as_ref()))?;
            }
            Type::Deref(inner) => {
                f.write_fmt(format_args!("Dr{}", inner.as_ref()))?;
            }
            Type::Parameterized(id, others) => {
                f.write_fmt(format_args!("Par{}", id))?;
                f.write_str("W")?;
                for other in others {
                    other.fmt(f)?;
                }
            }
            Type::Void => {
                f.write_str("void")?;
            }
            Type::Unset => {
                f.write_str("INFER_ERR")?;
            }
            Type::Error => {
                f.write_str("BadType")?;
            }
            Type::Int32 => {
                f.write_str("int32_t")?;
            }
            Type::Float => {
                f.write_str("float")?;
            }
            Type::UnspecificNumeric => {
                f.write_str("NUMERIC_ERR")?;
            }
        }

        Ok(())
    }
}