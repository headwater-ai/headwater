// SPDX-License-Identifier: Apache-2.0
//! The meta-schema itself: the file, read into shapes, and the lookup that
//! turns an overlay address into the shape at the node it names.

use crate::error::{SchemaError, SchemaErrorKind};
use crate::shape::{Form, Member, Shape};
use headwater_ref::Address;
use headwater_yaml::{Options, Span, Value};
use std::collections::BTreeMap;

/// The meta-schema, as this crate ships it.
///
/// It is one file, published with the engine and versioned with it
/// ([spec 2](../../../../docs/spec/02-taxonomy-model.md#the-meta-schema)). The
/// text is compiled in rather than read from disk, because a validator that
/// looks for its own schema on a path has a second failure mode that says
/// nothing useful to an adopter.
#[derive(Debug)]
pub struct MetaSchema {
    name: String,
    version: String,
    declarations: Vec<Member>,
    operations: Vec<Member>,
    definitions: BTreeMap<String, Shape>,
}

/// The shipped source, so that a caller can read the file the engine validates
/// against rather than a description of it.
pub const SOURCE: &str = include_str!("../meta-schema.yml");

impl MetaSchema {
    /// The shipped meta-schema.
    pub fn shipped() -> Result<Self, SchemaError> {
        Self::read(SOURCE)
    }

    /// Read a meta-schema source. The dialect is the taxonomy dialect, which is
    /// the whole of what "expressed in its own dialect" buys: the file the
    /// validator reads goes through the same loader, on the same Q2 rulings, as
    /// the sources it validates.
    pub fn read(source: &str) -> Result<Self, SchemaError> {
        let root = headwater_yaml::load_with(source, Options::default()).map_err(|errors| {
            SchemaError::new(
                SchemaErrorKind::Unloadable(headwater_yaml::error::render(&errors)),
                "",
                Span::default(),
            )
        })?;
        let map = root
            .value
            .as_map()
            .ok_or_else(|| SchemaError::new(SchemaErrorKind::ShapeNotAMapping, "", root.span))?;

        let name = scalar(map, "meta_schema").unwrap_or_default();
        let version = scalar(map, "version").unwrap_or_default();

        let declarations = members(map, "declarations")?;
        let operations = members(map, "operations")?;

        let mut definitions = BTreeMap::new();
        if let Some(block) = map.get("definitions") {
            let entries = block.value.as_map().ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::ShapeNotAMapping, "definitions", block.span)
            })?;
            for entry in entries {
                let at = format!("definitions.{}", entry.key.value);
                let shape = Shape::read(&entry.value.value, entry.value.span, &at)?;
                definitions.insert(entry.key.value.clone(), shape);
            }
        }

        let schema = Self {
            name,
            version,
            declarations,
            operations,
            definitions,
        };
        schema.check_every_block_resolves()?;
        Ok(schema)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn declarations(&self) -> &[Member] {
        &self.declarations
    }

    pub fn operations(&self) -> &[Member] {
        &self.operations
    }

    /// Follow a `block:` to the shape it names.
    pub fn resolve<'a>(&'a self, shape: &'a Shape) -> &'a Shape {
        let mut current = shape;
        // A `block` that names a `block` is legal and terminates, because
        // `check_every_block_resolves` refuses a cycle at load.
        while let Form::Block(name) = &current.form {
            match self.definitions.get(name) {
                Some(next) => current = next,
                None => return current,
            }
        }
        current
    }

    /// The shape at the node an overlay address names.
    ///
    /// This is the half of the resolver's work that needs no tree. An address
    /// either lands on a declared position or it does not, and whether it
    /// reaches into a list is decided here rather than by reading a merged
    /// tree. [Spec 2](../../../../docs/spec/02-taxonomy-model.md#the--reference-sublanguage)
    /// puts the list refusal outside the grammar because `0` is a legal key
    /// name, and this is where the refusal lands instead.
    pub fn at(&self, address: &Address) -> Position<'_> {
        let segments = address.segments();
        let Some(member) = self
            .declarations
            .iter()
            .find(|member| member.name == segments[0])
        else {
            return Position::Undeclared {
                stopped_at: segments[0].clone(),
            };
        };
        self.walk(&member.shape, &segments[1..], &segments[0..1])
    }

    fn walk<'a>(&'a self, shape: &'a Shape, rest: &[String], seen: &[String]) -> Position<'a> {
        let shape = self.resolve(shape);
        let Some((head, tail)) = rest.split_first() else {
            return Position::At(shape);
        };
        let here = seen.join(".");
        match &shape.form {
            Form::Map(value) => {
                let mut deeper = seen.to_vec();
                deeper.push(head.clone());
                self.walk(value, tail, &deeper)
            }
            Form::Members(members) => match members.iter().find(|member| member.name == *head) {
                Some(member) => {
                    let mut deeper = seen.to_vec();
                    deeper.push(head.clone());
                    self.walk(&member.shape, tail, &deeper)
                }
                None => Position::Undeclared {
                    stopped_at: head.clone(),
                },
            },
            Form::Seq(_) => Position::InsideList { list: here },
            Form::Scalar(_) | Form::Enum(_) => Position::BelowScalar { scalar: here },
            Form::Free(_) => Position::At(shape),
            Form::OneOf(alternatives) => {
                for alternative in alternatives {
                    let found = self.walk(alternative, rest, seen);
                    if matches!(found, Position::At(_)) {
                        return found;
                    }
                }
                Position::Undeclared {
                    stopped_at: head.clone(),
                }
            }
            Form::Block(name) => Position::Undeclared {
                stopped_at: name.clone(),
            },
        }
    }

    /// Every `block:` names a definition, and following one terminates.
    ///
    /// Two passes, because there are two failures and they are not the same
    /// one. A `block:` that reaches itself through `block:` alone consumes no
    /// node on the way, so [`MetaSchema::resolve`] would loop forever inside a
    /// validator that an adopter runs. A `block:` that reaches itself through a
    /// `seq`, a `map` or a `members` is an ordinary recursive shape, and it
    /// terminates because a node has to be there for the walk to go on.
    fn check_every_block_resolves(&self) -> Result<(), SchemaError> {
        for (name, shape) in &self.definitions {
            let mut chain = vec![name.clone()];
            let mut current = shape;
            while let Form::Block(next) = &current.form {
                if chain.contains(next) {
                    return Err(SchemaError::new(
                        SchemaErrorKind::CyclicBlock(next.clone()),
                        name,
                        Span::default(),
                    ));
                }
                current = self.definitions.get(next).ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::UndefinedBlock(next.clone()),
                        name,
                        Span::default(),
                    )
                })?;
                chain.push(next.clone());
            }
        }

        let mut visited = std::collections::BTreeSet::new();
        for member in self.declarations.iter().chain(&self.operations) {
            self.check_defined(&member.shape, &member.name, &mut visited)?;
        }
        for (name, shape) in &self.definitions {
            self.check_defined(shape, name, &mut visited)?;
        }
        Ok(())
    }

    fn check_defined(
        &self,
        shape: &Shape,
        at: &str,
        visited: &mut std::collections::BTreeSet<String>,
    ) -> Result<(), SchemaError> {
        match &shape.form {
            Form::Block(name) => {
                let next = self.definitions.get(name).ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::UndefinedBlock(name.clone()),
                        at,
                        Span::default(),
                    )
                })?;
                if visited.insert(name.clone()) {
                    self.check_defined(next, name, visited)?;
                }
                Ok(())
            }
            Form::Seq(inner) | Form::Map(inner) => self.check_defined(inner, at, visited),
            Form::Members(members) => {
                for member in members {
                    self.check_defined(&member.shape, &member.name, visited)?;
                }
                Ok(())
            }
            Form::OneOf(alternatives) => {
                for alternative in alternatives {
                    self.check_defined(alternative, at, visited)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Every `free` in the meta-schema, with the reason it carries.
    ///
    /// A gap that the specification has not closed is data here rather than a
    /// comment, so that a reader can count what the meta-schema declines to
    /// constrain instead of taking the crate's word for the number.
    pub fn unconstrained(&self) -> Vec<(String, String)> {
        let mut found = Vec::new();
        for member in self.declarations.iter().chain(&self.operations) {
            collect_free(&member.shape, &member.name, &mut found);
        }
        for (name, shape) in &self.definitions {
            collect_free(shape, name, &mut found);
        }
        found.sort();
        found.dedup();
        found
    }
}

fn collect_free(shape: &Shape, at: &str, found: &mut Vec<(String, String)>) {
    match &shape.form {
        Form::Free(reason) => found.push((at.to_string(), reason.clone())),
        Form::Seq(inner) | Form::Map(inner) => collect_free(inner, at, found),
        Form::Members(members) => {
            for member in members {
                collect_free(&member.shape, &format!("{at}.{}", member.name), found);
            }
        }
        Form::OneOf(alternatives) => {
            for alternative in alternatives {
                collect_free(alternative, at, found);
            }
        }
        _ => {}
    }
}

/// What an address lands on.
#[derive(Debug)]
pub enum Position<'a> {
    At(&'a Shape),
    Undeclared { stopped_at: String },
    InsideList { list: String },
    BelowScalar { scalar: String },
}

fn scalar(map: &headwater_yaml::Mapping, key: &str) -> Option<String> {
    map.get(key)
        .and_then(|value| value.value.as_scalar())
        .map(|scalar| scalar.text.clone())
}

fn members(map: &headwater_yaml::Mapping, block: &'static str) -> Result<Vec<Member>, SchemaError> {
    let entry = map.get(block).ok_or_else(|| {
        SchemaError::new(SchemaErrorKind::MissingBlock(block), block, Span::default())
    })?;
    let entries = entry
        .value
        .as_map()
        .ok_or_else(|| SchemaError::new(SchemaErrorKind::ShapeNotAMapping, block, entry.span))?;
    let mut read = Vec::new();
    for item in entries {
        let name = item.key.value.clone();
        let at = format!("{block}.{name}");
        let shape = Shape::read(&item.value.value, item.value.span, &at)?;
        let required = item
            .value
            .value
            .as_map()
            .and_then(|map| headwater_yaml::core_schema::flag(map, "required"))
            .unwrap_or(false);
        read.push(Member {
            name,
            shape,
            required,
        });
    }
    Ok(read)
}

/// Whether `value` is a mapping, sequence or scalar, for a message.
pub fn form_of(value: &Value) -> &'static str {
    value.kind_name()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::Form;

    const HEAD: &str = "meta_schema: test\nversion: 0.0.0\noperations: {}\n";

    fn read(declarations: &str, definitions: &str) -> Result<MetaSchema, SchemaError> {
        let definitions = if definitions.is_empty() {
            String::new()
        } else {
            format!("definitions:\n{definitions}")
        };
        MetaSchema::read(&format!("{HEAD}declarations:\n{declarations}{definitions}"))
    }

    #[test]
    fn the_shipped_meta_schema_loads() {
        let schema = MetaSchema::shipped().expect("the shipped meta-schema");
        assert_eq!(schema.name(), "headwater/taxonomy");
        // Thirteen declarations, `vocabularies`, and the three that name the
        // taxonomy and what it extends.
        assert_eq!(schema.declarations().len(), 17);
    }

    #[test]
    fn a_block_that_names_nothing_is_refused_when_the_meta_schema_loads() {
        let error = read("  kinds: {block: kind}\n", "  facet: {scalar: string}\n")
            .expect_err("an undefined block");
        assert!(matches!(error.kind, SchemaErrorKind::UndefinedBlock(name) if name == "kind"));
    }

    /// A definition that names itself would hang [`MetaSchema::resolve`], and
    /// that loop sits inside a validator an adopter runs.
    #[test]
    fn a_block_that_names_itself_is_refused_when_the_meta_schema_loads() {
        let error = read("  kinds: {block: kind}\n", "  kind: {block: kind}\n")
            .expect_err("a cycle of blocks");
        assert!(matches!(error.kind, SchemaErrorKind::CyclicBlock(name) if name == "kind"));
    }

    /// A cycle that goes through a `map` is not the same thing, because a node
    /// has to be there for the walk to continue.
    #[test]
    fn a_block_reached_through_a_map_may_name_itself() {
        let schema = read(
            "  kinds: {block: kind}\n",
            "  kind: {members: {children: {map: {block: kind}}}}\n",
        )
        .expect("a schema");
        let shape = &schema.declarations()[0].shape;
        assert!(matches!(schema.resolve(shape).form, Form::Members(_)));
    }

    #[test]
    fn a_shape_carries_one_form_and_the_file_is_refused_if_it_carries_two() {
        let error =
            read("  kinds: {map: {scalar: string}, scalar: string}\n", "").expect_err("two forms");
        assert!(matches!(error.kind, SchemaErrorKind::TwoForms { .. }));
    }

    #[test]
    fn an_address_below_a_map_reaches_the_value_shape() {
        let schema = read(
            "  shelves: {map: {members: {path: {scalar: string}}}}\n",
            "",
        )
        .expect("a schema");
        let address = Address::parse("shelves.decisions.path").expect("an address");
        assert!(matches!(schema.at(&address), Position::At(_)));
        let missing = Address::parse("shelves.decisions.depth").expect("an address");
        assert!(matches!(schema.at(&missing), Position::Undeclared { .. }));
    }
}
