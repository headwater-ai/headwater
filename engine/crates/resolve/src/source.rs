// SPDX-License-Identifier: Apache-2.0
//! A source on its way into a resolution: loaded, named, and validated against
//! the meta-schema before anything merges it.
//!
//! [Spec 2](../../../../docs/spec/02-taxonomy-model.md#the-meta-schema): "The
//! engine never applies a taxonomy that does not validate. There is no partial
//! load mode." So validation is here rather than in a verb that a caller may
//! forget to run, and `headwater taxonomy validate`
//! ([#51](https://github.com/headwater-ai/headwater/issues/51)) is the same
//! check with a report in front of it.

use crate::error::{ResolveError, ResolveErrorKind};
use headwater_meta::MetaSchema;
use headwater_yaml::{Span, Spanned, Value};
use std::path::Path;

/// Which root set a source is read against.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// Declarations: a base package or any taxonomy written out whole.
    Taxonomy,
    /// Operations at addresses: a bundle or an adopter overlay.
    Overlay,
}

/// One source, with the name a message calls it by.
#[derive(Clone, Debug)]
pub struct Source {
    /// The path as a reader of the repository would write it.
    pub name: String,
    pub role: Role,
    pub root: Spanned<Value>,
    /// The text this loaded from. The lock records a digest of it, so that a
    /// stale lock can name the file that moved without resolving anything.
    pub text: String,
    /// The bundle name this source was selected as: the name in a consumer's
    /// `bundles:` list, or the directory name a package ships it under. `None`
    /// for a base, an adopter overlay, and any source nothing selected as a
    /// bundle.
    ///
    /// The name comes from the selection and never from the source's own
    /// `bundle:` key. [HW-DR-0095] reads `requires` for one purpose and leaves
    /// the other keys of Q40 as labels, so the key a bundle writes about itself
    /// stays unread. The resolver reads this source's `requires` only when this
    /// is `Some`.
    ///
    /// [HW-DR-0095]: ../../../../docs/decisions/0095-q67-one-library-entry-may-address-the-keys-of-an-entry-it-names-in-requires-and-confluence-holds-over-the-dependency-order.md
    pub selected_as: Option<String>,
}

impl Source {
    /// This source, named as the bundle a selection chose.
    #[must_use]
    pub fn selected_as(mut self, bundle: &str) -> Self {
        self.selected_as = Some(bundle.to_string());
        self
    }

    pub fn read(path: &Path, name: &str, role: Role) -> Result<Self, Vec<ResolveError>> {
        let text = read_text(path).map_err(|mut errors| {
            for error in &mut errors {
                error.source = name.to_string();
            }
            errors
        })?;
        Self::from_text(name, role, &text)
    }

    /// A source read from text, for a caller that holds one already.
    pub fn from_text(name: &str, role: Role, source: &str) -> Result<Self, Vec<ResolveError>> {
        let root = headwater_yaml::load(source).map_err(|errors| {
            vec![ResolveError::new(
                ResolveErrorKind::SourceRefused(headwater_yaml::error::render(&errors)),
                name,
                "",
                Span::default(),
            )]
        })?;
        Ok(Self {
            name: name.to_string(),
            role,
            root,
            text: source.to_string(),
            selected_as: None,
        })
    }

    /// What the meta-schema makes of this source.
    pub fn validate(&self, schema: &MetaSchema) -> Vec<ResolveError> {
        let found = match self.role {
            Role::Taxonomy => headwater_meta::validate::taxonomy(schema, &self.root),
            Role::Overlay => headwater_meta::validate::overlay(schema, &self.root),
        };
        found
            .into_iter()
            .map(|error| {
                // The kind rather than the whole error, because a `ResolveError`
                // prints its own source, position and operation, and a message
                // that stated the position twice would read as two findings.
                ResolveError::new(
                    ResolveErrorKind::SourceRefused(error.kind.to_string()),
                    &self.name,
                    &error.at,
                    error.span,
                )
            })
            .collect()
    }
}

/// The text of one file, with a refusal that names it.
///
/// Only a regular file is opened. `metadata` follows a link, so a source
/// reached through one still reads. A named pipe, a socket or a device is
/// refused before it is opened, because a pipe with no writer blocks its reader
/// for ever. This is the one reader under the consumer declaration, the
/// overlay and every package source (#1366).
fn read_text(path: &Path) -> Result<String, Vec<ResolveError>> {
    let name = path.display().to_string();
    let refused = |why: String| {
        vec![ResolveError::new(
            ResolveErrorKind::SourceRefused(format!("cannot read {name}: {why}")),
            &name,
            "",
            Span::default(),
        )]
    };
    if std::fs::metadata(path).is_ok_and(|meta| !meta.is_file()) {
        return Err(refused("it is not a regular file".to_string()));
    }
    std::fs::read_to_string(path).map_err(|error| refused(error.to_string()))
}

/// Load one YAML file on the taxonomy dialect.
pub fn load(path: &Path) -> Result<Spanned<Value>, Vec<ResolveError>> {
    let name = path.display().to_string();
    let source = read_text(path)?;
    headwater_yaml::load(&source).map_err(|errors| {
        vec![ResolveError::new(
            ResolveErrorKind::SourceRefused(headwater_yaml::error::render(&errors)),
            &name,
            "",
            Span::default(),
        )]
    })
}
