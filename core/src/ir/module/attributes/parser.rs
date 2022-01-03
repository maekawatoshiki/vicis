use super::Attribute;
use crate::ir::util::{spaces, string_literal};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::map,
    error::VerboseError,
    multi::many0,
    sequence::{preceded, tuple},
    IResult,
};

pub fn parse_attribute(source: &str) -> IResult<&str, Attribute, VerboseError<&str>> {
    alt((
        alt((
            map(tag("alwaysinline"), |_| Attribute::AlwaysInline),
            map(tag("builtin"), |_| Attribute::Builtin),
            map(tag("cold"), |_| Attribute::Cold),
            map(tag("convergent"), |_| Attribute::Convergent),
            map(tag("inaccessiblememonly"), |_| {
                Attribute::InaccessibleMemOnly
            }),
            map(tag("inaccessiblememorargmemonly"), |_| {
                Attribute::InaccessibleMemOrArgMemOnly
            }),
            map(tag("inlinehint"), |_| Attribute::InlineHint),
            map(tag("jumptable"), |_| Attribute::JumpTable),
            map(tag("minimizesize"), |_| Attribute::MinimizeSize),
            map(tag("naked"), |_| Attribute::Naked),
            map(tag("nobuiltin"), |_| Attribute::NoBuiltin),
            map(tag("nocfcheck"), |_| Attribute::NoCFCheck),
        )),
        alt((
            map(tag("noduplicate"), |_| Attribute::NoDuplicate),
            map(tag("nofree"), |_| Attribute::NoFree),
            map(tag("noimplicitfloat"), |_| Attribute::NoImplicitFloat),
            map(tag("noinline"), |_| Attribute::NoInline),
            map(tag("nonlazybind"), |_| Attribute::NonLazyBind),
            map(tag("noredzone"), |_| Attribute::NoRedZone),
            map(tag("noreturn"), |_| Attribute::NoReturn),
            map(tag("norecurse"), |_| Attribute::NoRecurse),
            map(tag("willreturn"), |_| Attribute::WillReturn),
            map(tag("returnstwice"), |_| Attribute::ReturnsTwice),
            map(tag("nosync"), |_| Attribute::NoSync),
            map(tag("nounwind"), |_| Attribute::NoUnwind),
            map(tag("optforfuzzing"), |_| Attribute::OptForFuzzing),
            map(tag("optnone"), |_| Attribute::OptNone),
            map(tag("optsize"), |_| Attribute::OptSize),
            map(tag("readnone"), |_| Attribute::ReadNone),
            map(tag("readonly"), |_| Attribute::ReadOnly),
            map(tag("writeonly"), |_| Attribute::WriteOnly),
            map(tag("argmemonly"), |_| Attribute::ArgMemOnly),
        )),
        alt((
            map(tag("safestack"), |_| Attribute::SafeStack),
            map(tag("sanitizeaddress"), |_| Attribute::SanitizeAddress),
            map(tag("sanitizememory"), |_| Attribute::SanitizeMemory),
            map(tag("sanitizethread"), |_| Attribute::SanitizeThread),
            map(tag("sanitizehwaddress"), |_| Attribute::SanitizeHWAddress),
            map(tag("sanitizememtag"), |_| Attribute::SanitizeMemTag),
            map(tag("shadowcallstack"), |_| Attribute::ShadowCallStack),
            map(tag("speculativeloadhardening"), |_| {
                Attribute::SpeculativeLoadHardening
            }),
            map(tag("speculatable"), |_| Attribute::Speculatable),
            map(tag("ssp"), |_| Attribute::StackProtect),
            map(tag("sspreq"), |_| Attribute::StackProtectReq),
            map(tag("sspstrong"), |_| Attribute::StackProtectStrong),
            map(tag("strictfp"), |_| Attribute::StrictFP),
            map(tag("uwtable"), |_| Attribute::UWTable),
            // map(tag("unknownattribute"), |_| Attribute::UnknownAttribute),
            map(
                tuple((string_literal, spaces, char('='), spaces, string_literal)),
                |(kind, _, _, _, value)| Attribute::StringAttribute { kind, value },
            ),
            map(string_literal, |kind| Attribute::StringAttribute {
                kind,
                value: "".to_string(),
            }),
            map(preceded(char('#'), digit1), |num: &str| {
                Attribute::Ref(num.parse::<u32>().unwrap())
            }),
        )),
    ))(source)
}

pub fn parse_attributes(source: &str) -> IResult<&str, Vec<Attribute>, VerboseError<&str>> {
    many0(preceded(spaces, parse_attribute))(source)
}
