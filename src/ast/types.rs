use std::fmt::{Display, Formatter};

use std::io::Write;
use crate::string_pool::PoolS;

pub enum Type {
    Primitive(PoolS),
    Optional(Box<Type>),
    Deref(Box<Type>),
    Parameterized(PoolS, Vec<Type>),
    Void,
    Error
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
            Type::Error => {
                f.write_str("BadType")?;
            }
        }

        Ok(())
    }
}