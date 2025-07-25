use crate::{kw, sol_path, SolPath, Spanned};
use proc_macro2::Span;
use std::{
    fmt,
    fmt::Write,
    hash::{Hash, Hasher},
    num::{IntErrorKind, NonZeroU16},
};
use syn::{
    ext::IdentExt,
    parse::{Lookahead1, Parse, ParseStream},
    token::{Bracket, Paren},
    Error, Ident, Result, Token,
};

mod array;
pub use array::TypeArray;

mod function;
pub use function::TypeFunction;

mod mapping;
pub use mapping::TypeMapping;

mod tuple;
pub use tuple::TypeTuple;

/// A type name.
///
/// Solidity reference:
/// <https://docs.soliditylang.org/en/latest/grammar.html#a4.SolidityParser.typeName>
#[derive(Clone)]
pub enum Type {
    // TODO: `fixed` and `ufixed`
    /// `address $(payable)?`
    Address(Span, Option<kw::payable>),
    /// `bool`
    Bool(Span),
    /// `string`
    String(Span),

    /// `bytes`
    Bytes(Span),
    /// `bytes<size>`
    FixedBytes(Span, NonZeroU16),

    /// `int[size]`
    Int(Span, Option<NonZeroU16>),
    /// `uint[size]`
    Uint(Span, Option<NonZeroU16>),

    #[cfg(feature = "seismic")]
    /// `sint[size]`
    Sint(Span, Option<NonZeroU16>),
    #[cfg(feature = "seismic")]
    /// `suint[size]`
    Suint(Span, Option<NonZeroU16>),
    #[cfg(feature = "seismic")]
    /// `saddress`
    Saddress(Span),
    #[cfg(feature = "seismic")]
    /// `sbool`
    Sbool(Span),

    /// `$ty[$($size)?]`
    Array(TypeArray),
    /// `$(tuple)? ( $($types,)* )`
    Tuple(TypeTuple),
    /// `function($($arguments),*) $($attributes)* $(returns ($($returns),+))?`
    Function(TypeFunction),
    /// `mapping($key $($key_name)? => $value $($value_name)?)`
    Mapping(TypeMapping),

    /// A custom type.
    Custom(SolPath),
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Address(..), Self::Address(..)) => true,
            (Self::Bool(_), Self::Bool(_)) => true,
            (Self::String(_), Self::String(_)) => true,
            (Self::Bytes { .. }, Self::Bytes { .. }) => true,

            (Self::FixedBytes(_, a), Self::FixedBytes(_, b)) => a == b,
            (Self::Int(_, a), Self::Int(_, b)) => a == b,
            (Self::Uint(_, a), Self::Uint(_, b)) => a == b,

            #[cfg(feature = "seismic")]
            (Self::Sint(_, a), Self::Sint(_, b)) => a == b,
            #[cfg(feature = "seismic")]
            (Self::Suint(_, a), Self::Suint(_, b)) => a == b,
            #[cfg(feature = "seismic")]
            (Self::Saddress(_), Self::Saddress(_)) => true,
            #[cfg(feature = "seismic")]
            (Self::Sbool(_), Self::Sbool(_)) => true,

            (Self::Tuple(a), Self::Tuple(b)) => a == b,
            (Self::Array(a), Self::Array(b)) => a == b,
            (Self::Function(a), Self::Function(b)) => a == b,
            (Self::Mapping(a), Self::Mapping(b)) => a == b,
            (Self::Custom(a), Self::Custom(b)) => a == b,

            _ => false,
        }
    }
}

impl Eq for Type {}

impl Hash for Type {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Address(..) | Self::Bool(_) | Self::String(_) | Self::Bytes(_) => {}

            Self::FixedBytes(_, size) => size.hash(state),
            Self::Int(_, size) => size.hash(state),
            Self::Uint(_, size) => size.hash(state),

            #[cfg(feature = "seismic")]
            Self::Sint(_, size) => size.hash(state),
            #[cfg(feature = "seismic")]
            Self::Suint(_, size) => size.hash(state),
            #[cfg(feature = "seismic")]
            Self::Saddress(_) | Self::Sbool(_) => {}

            Self::Tuple(tuple) => tuple.hash(state),
            Self::Array(array) => array.hash(state),
            Self::Function(function) => function.hash(state),
            Self::Mapping(mapping) => mapping.hash(state),
            Self::Custom(custom) => custom.hash(state),
        }
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Type::")?;
        match self {
            Self::Address(_, None) => f.write_str("Address"),
            Self::Address(_, Some(_)) => f.write_str("AddressPayable"),
            Self::Bool(_) => f.write_str("Bool"),
            Self::String(_) => f.write_str("String"),
            Self::Bytes(_) => f.write_str("Bytes"),

            Self::FixedBytes(_, size) => f.debug_tuple("FixedBytes").field(size).finish(),
            Self::Int(_, size) => f.debug_tuple("Int").field(size).finish(),
            Self::Uint(_, size) => f.debug_tuple("Uint").field(size).finish(),

            #[cfg(feature = "seismic")]
            Self::Sint(_, size) => f.debug_tuple("Sint").field(size).finish(),
            #[cfg(feature = "seismic")]
            Self::Suint(_, size) => f.debug_tuple("Suint").field(size).finish(),
            #[cfg(feature = "seismic")]
            Self::Saddress(_) => f.write_str("Saddress"),
            #[cfg(feature = "seismic")]
            Self::Sbool(_) => f.write_str("Sbool"),

            Self::Tuple(tuple) => tuple.fmt(f),
            Self::Array(array) => array.fmt(f),
            Self::Function(function) => function.fmt(f),
            Self::Mapping(mapping) => mapping.fmt(f),
            Self::Custom(custom) => f.debug_tuple("Custom").field(custom).finish(),
        }
    }
}

/// Canonical type name formatting, used in selector preimages.
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Address(_, _) => f.write_str("address"),
            Self::Bool(_) => f.write_str("bool"),
            Self::String(_) => f.write_str("string"),
            Self::Bytes(_) => f.write_str("bytes"),

            Self::FixedBytes(_, size) => write!(f, "bytes{size}"),
            Self::Int(_, size) => write_opt(f, "int", *size),
            Self::Uint(_, size) => write_opt(f, "uint", *size),

            #[cfg(feature = "seismic")]
            Self::Sint(_, size) => write_opt(f, "sint", *size),
            #[cfg(feature = "seismic")]
            Self::Suint(_, size) => write_opt(f, "suint", *size),
            #[cfg(feature = "seismic")]
            Self::Saddress(_) => f.write_str("saddress"),
            #[cfg(feature = "seismic")]
            Self::Sbool(_) => f.write_str("sbool"),

            Self::Tuple(tuple) => tuple.fmt(f),
            Self::Array(array) => array.fmt(f),
            Self::Function(_) => f.write_str("function"),
            Self::Mapping(mapping) => mapping.fmt(f),
            Self::Custom(custom) => custom.fmt(f),
        }
    }
}

impl Parse for Type {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut candidate = Self::parse_simple(input)?;

        // while the next token is a bracket, parse an array size and nest the
        // candidate into an array
        while input.peek(Bracket) {
            candidate = Self::Array(TypeArray::parse_nested(Box::new(candidate), input)?);
        }

        Ok(candidate)
    }
}

impl Spanned for Type {
    fn span(&self) -> Span {
        match self {
            &Self::Address(span, payable) => {
                payable.and_then(|kw| span.join(kw.span)).unwrap_or(span)
            }
            Self::Bool(span)
            | Self::String(span)
            | Self::Bytes(span)
            | Self::FixedBytes(span, _)
            | Self::Int(span, _)
            | Self::Uint(span, _) => *span,
            #[cfg(feature = "seismic")]
            Self::Sint(span, _)
            | Self::Suint(span, _)
            | Self::Saddress(span)
            | Self::Sbool(span) => *span,
            Self::Tuple(tuple) => tuple.span(),
            Self::Array(array) => array.span(),
            Self::Function(function) => function.span(),
            Self::Mapping(mapping) => mapping.span(),
            Self::Custom(custom) => custom.span(),
        }
    }

    fn set_span(&mut self, new_span: Span) {
        match self {
            Self::Address(span, payable) => {
                *span = new_span;
                if let Some(kw) = payable {
                    kw.span = new_span;
                }
            }
            Self::Bool(span)
            | Self::String(span)
            | Self::Bytes(span)
            | Self::FixedBytes(span, _)
            | Self::Int(span, _)
            | Self::Uint(span, _) => *span = new_span,

            #[cfg(feature = "seismic")]
            Self::Sint(span, _)
            | Self::Suint(span, _)
            | Self::Saddress(span)
            | Self::Sbool(span) => *span = new_span,

            Self::Tuple(tuple) => tuple.set_span(new_span),
            Self::Array(array) => array.set_span(new_span),
            Self::Function(function) => function.set_span(new_span),
            Self::Mapping(mapping) => mapping.set_span(new_span),
            Self::Custom(custom) => custom.set_span(new_span),
        }
    }
}

impl Type {
    pub fn custom(ident: Ident) -> Self {
        Self::Custom(sol_path![ident])
    }

    pub fn peek(lookahead: &Lookahead1<'_>) -> bool {
        lookahead.peek(syn::token::Paren)
            || lookahead.peek(kw::tuple)
            || lookahead.peek(kw::function)
            || lookahead.peek(kw::mapping)
            || lookahead.peek(Ident::peek_any)
    }

    /// Parses an identifier as an [elementary type name][ref].
    ///
    /// Note that you will have to check for the existence of a `payable`
    /// keyword separately.
    ///
    /// [ref]: https://docs.soliditylang.org/en/latest/grammar.html#a4.SolidityParser.elementaryTypeName
    pub fn parse_ident(ident: Ident) -> Self {
        Self::try_parse_ident(ident.clone()).unwrap_or_else(|_| Self::custom(ident))
    }

    pub fn try_parse_ident(ident: Ident) -> Result<Self> {
        let span = ident.span();
        let s = ident.to_string();
        let ret = match s.as_str() {
            "address" => Self::Address(span, None),
            "bool" => Self::Bool(span),
            "string" => Self::String(span),
            #[cfg(feature = "seismic")]
            "saddress" => Self::Saddress(span),
            #[cfg(feature = "seismic")]
            "sbool" => Self::Sbool(span),
            s => {
                #[cfg(feature = "seismic")]
                {
                    let seismic_type = if let Some(s) = s.strip_prefix("sint") {
                        match parse_size(s, span)? {
                            None => None,
                            Some(Some(size)) if size.get() > 256 || size.get() % 8 != 0 => {
                                return Err(Error::new(
                                    span,
                                    "sintX must be a multiple of 8 up to 256",
                                ))
                            }
                            Some(size) => Some(Self::Sint(span, size)),
                        }
                    } else if let Some(s) = s.strip_prefix("suint") {
                        match parse_size(s, span)? {
                            None => None,
                            Some(Some(size)) if size.get() > 256 || size.get() % 8 != 0 => {
                                return Err(Error::new(
                                    span,
                                    "suintX must be a multiple of 8 up to 256",
                                ))
                            }
                            Some(size) => Some(Self::Suint(span, size)),
                        }
                    } else {
                        None
                    };
                    if let Some(ty) = seismic_type {
                        return Ok(ty);
                    }
                }

                if let Some(s) = s.strip_prefix("bytes") {
                    match parse_size(s, span)? {
                        None => Self::custom(ident),
                        Some(Some(size)) if size.get() > 32 => {
                            return Err(Error::new(span, "fixed bytes range is 1-32"))
                        }
                        Some(Some(size)) => Self::FixedBytes(span, size),
                        Some(None) => Self::Bytes(span),
                    }
                } else if let Some(s) = s.strip_prefix("int") {
                    match parse_size(s, span)? {
                        None => Self::custom(ident),
                        Some(Some(size)) if size.get() > 256 || size.get() % 8 != 0 => {
                            return Err(Error::new(span, "intX must be a multiple of 8 up to 256"))
                        }
                        Some(size) => Self::Int(span, size),
                    }
                } else if let Some(s) = s.strip_prefix("uint") {
                    match parse_size(s, span)? {
                        None => Self::custom(ident),
                        Some(Some(size)) if size.get() > 256 || size.get() % 8 != 0 => {
                            return Err(Error::new(span, "uintX must be a multiple of 8 up to 256"))
                        }
                        Some(size) => Self::Uint(span, size),
                    }
                } else {
                    Self::custom(ident)
                }
            }
        };
        Ok(ret)
    }

    /// Parses the `payable` keyword from the input stream if this type is an
    /// address.
    pub fn parse_payable(mut self, input: ParseStream<'_>) -> Self {
        if let Self::Address(_, opt @ None) = &mut self {
            *opt = input.parse().unwrap();
        }
        self
    }

    /// Returns whether this type is ABI-encoded as a single EVM word (32
    /// bytes).
    pub const fn is_one_word(&self) -> bool {
        #[cfg(feature = "seismic")]
        {
            let is_seismic_one_word = matches!(
                self,
                Self::Saddress(_) | Self::Sint(..) | Self::Suint(..) | Self::Sbool(_)
            );
            if is_seismic_one_word {
                return true;
            }
        }

        matches!(
            self,
            Self::Bool(_)
                | Self::Int(..)
                | Self::Uint(..)
                | Self::FixedBytes(..)
                | Self::Address(..)
                | Self::Function(_)
        )
    }

    /// Returns whether this type is dynamic according to ABI rules.
    ///
    /// Note that this does not account for custom types, such as UDVTs.
    pub fn is_abi_dynamic(&self) -> bool {
        match self {
            Self::Bool(_)
            | Self::Int(..)
            | Self::Uint(..)
            | Self::FixedBytes(..)
            | Self::Address(..)
            | Self::Function(_) => false,

            #[cfg(feature = "seismic")]
            Self::Sint(..) | Self::Suint(..) | Self::Saddress(..) | Self::Sbool(_) => false,

            Self::String(_) | Self::Bytes(_) | Self::Custom(_) => true,

            Self::Array(array) => array.is_abi_dynamic(),
            Self::Tuple(tuple) => tuple.is_abi_dynamic(),

            // not applicable
            Self::Mapping(_) => true,
        }
    }

    /// Returns whether this type is a value type.
    ///
    /// These types' variables are always passed by value.
    ///
    /// `custom_is_value_type` accounts for custom value types.
    ///
    /// See the [Solidity docs](https://docs.soliditylang.org/en/latest/types.html#value-types) for more information.
    pub fn is_value_type(&self, custom_is_value_type: impl Fn(&SolPath) -> bool) -> bool {
        match self {
            Self::Custom(custom) => custom_is_value_type(custom),
            _ => self.is_value_type_simple(),
        }
    }

    /// Returns whether this type is a simple value type.
    ///
    /// See [`is_value_type`](Self::is_value_type) for more information.
    pub fn is_value_type_simple(&self) -> bool {
        matches!(
            self,
            Self::Bool(_)
                | Self::Int(..)
                | Self::Uint(..)
                | Self::FixedBytes(..)
                | Self::Address(..)
                | Self::Function(_)
        )
    }

    pub const fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    pub const fn is_tuple(&self) -> bool {
        matches!(self, Self::Tuple(_))
    }

    pub const fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    /// Recurses into this type and returns whether it contains a custom type.
    pub fn has_custom(&self) -> bool {
        match self {
            Self::Custom(_) => true,
            Self::Array(a) => a.ty.has_custom(),
            Self::Tuple(t) => t.types.iter().any(Self::has_custom),
            Self::Function(f) => {
                f.arguments.iter().any(|arg| arg.ty.has_custom())
                    || f.returns
                        .as_ref()
                        .is_some_and(|ret| ret.returns.iter().any(|arg| arg.ty.has_custom()))
            }
            Self::Mapping(m) => m.key.has_custom() || m.value.has_custom(),
            Self::Bool(_)
            | Self::Int(..)
            | Self::Uint(..)
            | Self::FixedBytes(..)
            | Self::Address(..)
            | Self::String(_)
            | Self::Bytes(_) => false,
            #[cfg(feature = "seismic")]
            Self::Sint(..) | Self::Suint(..) | Self::Saddress(..) | Self::Sbool(_) => false,
        }
    }

    /// Same as [`has_custom`](Self::has_custom), but `Function` returns `false`
    /// rather than recursing into its arguments and return types.
    pub fn has_custom_simple(&self) -> bool {
        match self {
            Self::Custom(_) => true,
            Self::Array(a) => a.ty.has_custom_simple(),
            Self::Tuple(t) => t.types.iter().any(Self::has_custom_simple),
            Self::Mapping(m) => m.key.has_custom_simple() || m.value.has_custom_simple(),
            Self::Bool(_)
            | Self::Int(..)
            | Self::Uint(..)
            | Self::FixedBytes(..)
            | Self::Address(..)
            | Self::Function(_)
            | Self::String(_)
            | Self::Bytes(_) => false,
            #[cfg(feature = "seismic")]
            Self::Sint(..) | Self::Suint(..) | Self::Saddress(..) | Self::Sbool(_) => false,
        }
    }

    /// Returns the inner type.
    pub fn peel_arrays(&self) -> &Self {
        let mut this = self;
        while let Self::Array(array) = this {
            this = &array.ty;
        }
        this
    }

    /// Returns the Solidity ABI name for this type. This is `tuple` for custom types, otherwise the
    /// same as [`Display`](fmt::Display).
    pub fn abi_name(&self) -> String {
        let mut s = String::new();
        self.abi_name_raw(&mut s);
        s
    }

    /// Returns the Solidity ABI name for this type. This is `tuple` for custom types, otherwise the
    /// same as [`Display`](fmt::Display).
    pub fn abi_name_raw(&self, s: &mut String) {
        match self {
            Self::Custom(_) => s.push_str("tuple"),
            Self::Array(array) => {
                array.ty.abi_name_raw(s);
                if let Some(size) = array.size() {
                    write!(s, "[{size}]").unwrap();
                } else {
                    s.push_str("[]");
                }
            }
            _ => write!(s, "{self}").unwrap(),
        }
    }

    /// Traverses this type while calling `f`.
    #[cfg(feature = "visit")]
    pub fn visit(&self, f: impl FnMut(&Self)) {
        use crate::Visit;
        struct VisitType<F>(F);
        impl<F: FnMut(&Type)> Visit<'_> for VisitType<F> {
            fn visit_type(&mut self, ty: &Type) {
                (self.0)(ty);
                crate::visit::visit_type(self, ty);
            }
            // Reduce generated code size by explicitly implementing these methods as noops.
            fn visit_block(&mut self, _block: &crate::Block) {}
            fn visit_expr(&mut self, _expr: &crate::Expr) {}
            fn visit_stmt(&mut self, _stmt: &crate::Stmt) {}
            fn visit_file(&mut self, _file: &crate::File) {}
            fn visit_item(&mut self, _item: &crate::Item) {}
        }
        VisitType(f).visit_type(self);
    }

    /// Traverses this type while calling `f`.
    #[cfg(feature = "visit-mut")]
    pub fn visit_mut(&mut self, f: impl FnMut(&mut Self)) {
        use crate::VisitMut;
        struct VisitTypeMut<F>(F);
        impl<F: FnMut(&mut Type)> VisitMut<'_> for VisitTypeMut<F> {
            fn visit_type(&mut self, ty: &mut Type) {
                (self.0)(ty);
                crate::visit_mut::visit_type(self, ty);
            }
            // Reduce generated code size by explicitly implementing these methods as noops.
            fn visit_block(&mut self, _block: &mut crate::Block) {}
            fn visit_expr(&mut self, _expr: &mut crate::Expr) {}
            fn visit_stmt(&mut self, _stmt: &mut crate::Stmt) {}
            fn visit_file(&mut self, _file: &mut crate::File) {}
            fn visit_item(&mut self, _item: &mut crate::Item) {}
        }
        VisitTypeMut(f).visit_type(self);
    }

    /// Parses a type from the input, without recursing into arrays.
    #[inline]
    fn parse_simple(input: ParseStream<'_>) -> Result<Self> {
        if input.peek(Paren) || input.peek(kw::tuple) {
            input.parse().map(Self::Tuple)
        } else if input.peek(kw::function) {
            input.parse().map(Self::Function)
        } else if input.peek(kw::mapping) {
            input.parse().map(Self::Mapping)
        } else if input.peek2(Token![.]) {
            input.parse().map(Self::Custom)
        } else if input.peek(Ident::peek_any) {
            let ident = input.call(Ident::parse_any)?;
            Ok(Self::parse_ident(ident).parse_payable(input))
        } else {
            Err(input.error(
                "expected a Solidity type: \
                 `address`, `bool`, `string`, `bytesN`, `intN`, `uintN`, \
                 `tuple`, `function`, `mapping`, or a custom type name",
            ))
        }
    }
}

fn write_opt(f: &mut fmt::Formatter<'_>, name: &str, size: Option<NonZeroU16>) -> fmt::Result {
    f.write_str(name)?;
    if let Some(size) = size {
        write!(f, "{size}")?;
    }
    Ok(())
}

// None => Custom
// Some(size) => size
fn parse_size(s: &str, span: Span) -> Result<Option<Option<NonZeroU16>>> {
    let opt = match s.parse::<NonZeroU16>() {
        Ok(size) => Some(Some(size)),
        Err(e) => match e.kind() {
            // bytes
            IntErrorKind::Empty => Some(None),
            // bytes_
            IntErrorKind::InvalidDigit => None,
            // bytesN where N == 0 || N > MAX
            _ => return Err(Error::new(span, format_args!("invalid size: {e}"))),
        },
    };
    Ok(opt)
}
