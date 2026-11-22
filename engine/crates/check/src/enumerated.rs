// SPDX-License-Identifier: Apache-2.0
//! One declaration for a closed enum and the constant that lists it.
//!
//! Ten enums of the workspace carry an `ALL` constant, and a reader such as
//! `Format::parse` finds a variant through it. A hand-kept list let a new
//! variant compile and stay out of `ALL`, so the parser missed it and no test
//! failed (HW-OBL-0172). Rust has no stable way to count or list the variants
//! of an enum, and the workspace takes no dependency that derives one. So
//! [`crate::enumerated!`] writes the enum and its `ALL` from one list of
//! variants. A variant that `ALL` leaves out has to be written after a `;`,
//! where a reader sees that it is outside.
//!
//! It lives here because every crate that declares such an enum already
//! depends on `headwater-check`, so the macro adds no edge to the graph of
//! crates.

/// Declare an enum and an `ALL` constant, at the visibility of the enum, that
/// lists every unit variant before an optional `;` in declaration order.
///
/// The attributes and doc comments on the enum and on each variant pass
/// through. The attributes after the enum, before `const ALL;`, go on the
/// constant. A variant after the `;` is outside `ALL`. It may carry fields,
/// and its doc comment says why it stays out.
///
/// ```
/// headwater_check::enumerated! {
///     /// A color.
///     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
///     pub enum Color {
///         /// The first.
///         Red,
///         Green,
///         Blue;
///         /// A mix, which no loop over the colors meets.
///         Mix(u8),
///     }
///     /// Every color, in declaration order.
///     const ALL;
/// }
/// assert_eq!(Color::ALL, [Color::Red, Color::Green, Color::Blue]);
/// ```
#[macro_export]
macro_rules! enumerated {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $( $(#[$variant_meta:meta])* $variant:ident ),+ $(,)?
            $( ; $( $(#[$outside_meta:meta])* $outside:ident $( ( $($field:ty),* $(,)? ) )? ),+ $(,)? )?
        }
        $(#[$all_meta:meta])*
        const ALL;
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $( $(#[$variant_meta])* $variant, )+
            $( $( $(#[$outside_meta])* $outside $( ( $($field),* ) )?, )+ )?
        }

        impl $name {
            $(#[$all_meta])*
            $vis const ALL: [$name; [$(stringify!($variant)),+].len()] = [$($name::$variant),+];
        }
    };
}

#[cfg(test)]
mod tests {
    crate::enumerated! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum Three {
            /// A doc comment on a variant passes through.
            First,
            Second,
            Third;
            /// Outside `ALL`, with a field.
            Fourth(u32),
        }
        const ALL;
    }

    #[test]
    fn all_lists_every_unit_variant_before_the_semicolon_in_declaration_order() {
        assert_eq!(Three::ALL, [Three::First, Three::Second, Three::Third]);
        assert!(!Three::ALL.contains(&Three::Fourth(0)));
    }
}
