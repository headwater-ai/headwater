// SPDX-License-Identifier: Apache-2.0
//! Stub: the contract for #1086 before the rule exists.

use crate::instance::Outcome;
use crate::scope::{NeighbourhoodCheck, NeighbourhoodView};
use headwater_graph::Declarations;

pub const RULE: &str = "lifecycle.state.set_twice";

pub struct StateSetTwice<'a> {
    declarations: &'a Declarations,
}

impl<'a> StateSetTwice<'a> {
    pub fn over(declarations: &'a Declarations) -> Self {
        StateSetTwice { declarations }
    }
}

impl NeighbourhoodCheck for StateSetTwice<'_> {
    const RULE: &'static str = self::RULE;
    const VERSION: u32 = 1;

    fn instantiates(&self, _kind: &str) -> bool {
        !self.declarations.relations.is_empty()
    }

    fn evaluate(&self, _view: &NeighbourhoodView<'_>) -> Outcome {
        Outcome::Passed
    }
}
