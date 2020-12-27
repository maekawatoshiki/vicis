use super::Linkage;
use nom::{branch::alt, bytes::complete::tag, combinator::map, error::VerboseError, IResult};

pub fn parse<'a, 'b>(source: &'a str) -> IResult<&'a str, Linkage, VerboseError<&'a str>> {
    alt((
        map(tag("private"), |_| Linkage::Private),
        map(tag("internal"), |_| Linkage::Internal),
        map(tag("external"), |_| Linkage::External),
        map(tag("externalweak"), |_| Linkage::ExternalWeak),
        map(tag("availableexternally"), |_| Linkage::AvailableExternally),
        map(tag("linkonceany"), |_| Linkage::LinkOnceAny),
        map(tag("linkonceodr"), |_| Linkage::LinkOnceODR),
        map(tag("linkonceodrautohide"), |_| Linkage::LinkOnceODRAutoHide),
        map(tag("weakany"), |_| Linkage::WeakAny),
        map(tag("weakodr"), |_| Linkage::WeakODR),
        map(tag("common"), |_| Linkage::Common),
        map(tag("appending"), |_| Linkage::Appending),
        map(tag("dllimport"), |_| Linkage::DLLImport),
        map(tag("dllexport"), |_| Linkage::DLLExport),
        map(tag("ghost"), |_| Linkage::Ghost),
        map(tag("linkerprivate"), |_| Linkage::LinkerPrivate),
        map(tag("linkerprivateweak"), |_| Linkage::LinkerPrivateWeak),
    ))(source)
}
