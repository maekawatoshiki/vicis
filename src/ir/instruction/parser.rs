use super::super::{
    function::{Data, Layout},
    types::Types,
};
use nom::{error::VerboseError, IResult};

pub fn parse<'a>(
    source: &'a str,
    data: &mut Data,
    layout: &mut Layout,
    types: &Types,
) -> IResult<&'a str, (), VerboseError<&'a str>> {
    todo!()
}
