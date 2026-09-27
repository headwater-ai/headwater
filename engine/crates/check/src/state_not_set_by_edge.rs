// SPDX-License-Identifier: Apache-2.0
//! Stub, registered so the decisive fixture fails for its own reason.

use crate::instance::Outcome;
use crate::lifecycle_state::StateFacet;
use crate::scope::{EdgeCheck, EdgeUnit, EdgeView};
use crate::shape::Shape;
use headwater_graph::declarations::Relation;
use headwater_graph::Declarations;

pub const RULE: &str = "lifecycle.state.not_set_by_edge";

pub struct NotSetByEdge<'a> {
    setting: Vec<&'a Relation>,
    #[allow(dead_code)]
    shape: &'a Shape,
    facet: StateFacet,
}

impl<'a> NotSetByEdge<'a> {
    pub fn over(declarations: &'a Declarations, shape: &'a Shape) -> Self {
        NotSetByEdge {
            setting: declarations
                .relations
                .iter()
                .filter(|relation| relation.sets_target_state.is_some())
                .collect(),
            shape,
            facet: StateFacet::of(shape),
        }
    }
}

impl EdgeCheck for NotSetByEdge<'_> {
    const RULE: &'static str = self::RULE;
    const VERSION: u32 = 1;
    const UNIT: EdgeUnit = EdgeUnit::Pair;

    fn instantiates(&self, relation: &str) -> bool {
        self.facet.name.is_some() && self.setting.iter().any(|known| known.name == relation)
    }

    fn evaluate(&self, _view: &EdgeView<'_>) -> Outcome {
        Outcome::Passed
    }
}
