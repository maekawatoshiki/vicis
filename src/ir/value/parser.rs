use super::{
    super::{function::parser::ParserContext, types::TypeId},
    ValueId,
};
use nom::{error::VerboseError, IResult};

pub fn parse<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
    ty: TypeId,
) -> IResult<&'a str, ValueId, VerboseError<&'a str>> {
    todo!()
}
