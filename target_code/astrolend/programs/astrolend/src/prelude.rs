use anchor_lang::prelude::*;

pub type AstrolendResult<G = ()> = Result<G>;

pub use crate::{
    errors::AstrolendError,
    state::astrolend_group::{GroupConfig, AstrolendGroup},
};
