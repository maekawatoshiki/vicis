use crate::ir::module::linkage::Linkage;
use nom::{branch::alt, bytes::complete::tag, combinator::map, error::VerboseError, IResult};

pub fn parse(source: &str) -> IResult<&str, Linkage, VerboseError<&str>> {
    alt((
        map(tag("private"), |_| Linkage::Private),
        map(tag("internal"), |_| Linkage::Internal),
        map(tag("external"), |_| Linkage::External),
        map(tag("externalweak"), |_| Linkage::ExternalWeak),
        map(tag("availableexternally"), |_| Linkage::AvailableExternally),
        map(tag("linkonce_any"), |_| Linkage::LinkOnceAny),
        map(tag("linkonce_odr"), |_| Linkage::LinkOnceODR),
        map(tag("linkonce_odrautohide"), |_| {
            Linkage::LinkOnceODRAutoHide
        }),
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
