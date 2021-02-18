use super::super::{
    function::{
        basic_block::BasicBlockId,
        instruction,
        instruction::{Opcode, Operand},
        Data, Function, Layout, Parameter,
    },
    module::{attributes, name, preemption_specifier},
    types,
    types::Types,
    util::spaces,
    value::{Value, ValueId},
};
use either::Either;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, char, digit1},
    combinator::opt,
    error::VerboseError,
    sequence::{preceded, terminated, tuple},
    IResult,
};
use rustc_hash::FxHashMap;

// define [linkage] [PreemptionSpecifier] [visibility] [DLLStorageClass]
//        [cconv] [ret attrs]
//        <ResultType> @<FunctionName> ([argument list])
//        [(unnamed_addr|local_unnamed_addr)] [AddrSpace] [fn Attrs]
//        [section "name"] [comdat [($name)]] [align N] [gc] [prefix Constant]
//        [prologue Constant] [personality Constant] (!name !N)* { ... }

pub struct ParserContext<'a> {
    pub types: &'a Types,
    pub data: &'a mut Data,
    pub layout: &'a mut Layout,
    pub name_to_value: &'a mut FxHashMap<name::Name, ValueId>,
    pub name_to_block: &'a mut FxHashMap<name::Name, BasicBlockId>,
    pub cur_block: BasicBlockId,
}

pub fn parse_argument<'a>(
    source: &'a str,
    types: &Types,
    index: &mut usize,
) -> IResult<&'a str, Parameter, VerboseError<&'a str>> {
    let (source, ty) = types::parse(source, types)?;
    let (source, name) = opt(preceded(spaces, preceded(char('%'), name::parse)))(source)?;
    Ok((
        source,
        Parameter {
            name: name.unwrap_or(name::Name::Number({
                *index += 1;
                *index
            })),
            ty,
        },
    ))
}

pub fn parse_argument_list<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, Vec<Parameter>, VerboseError<&'a str>> {
    let (mut source, _) = tuple((spaces, char('(')))(source)?;

    if let Ok((source, _)) = tuple((spaces, char(')')))(source) {
        return Ok((source, vec![]));
    }

    let mut params = vec![];
    let mut index = 0;

    loop {
        let (source_, param) = parse_argument(source, types, &mut index)?;
        params.push(param);

        if let Ok((source_, _)) = tuple((spaces, char(',')))(source_) {
            source = source_;
            continue;
        }

        if let Ok((source, _)) = tuple((spaces, char(')')))(source_) {
            return Ok((source, params));
        }
    }
}
//
pub fn parse_func_attrs<'a>(
    mut source: &'a str,
) -> IResult<&'a str, Vec<Either<attributes::Attribute, u32>>, VerboseError<&'a str>> {
    let mut attrs = vec![];
    loop {
        if let Ok((source_, num)) = preceded(spaces, preceded(char('#'), digit1))(source) {
            attrs.push(Either::Right(num.parse::<u32>().unwrap()));
            source = source_;
            continue;
        }
        if let Ok((source_, attr)) = preceded(spaces, attributes::parser::parse_attribute)(source) {
            attrs.push(Either::Left(attr));
            source = source_;
        }
        break;
    }
    Ok((source, attrs))
}

pub fn parse_body<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
    num_args: usize,
) -> IResult<&'a str, (), VerboseError<&'a str>> {
    let (mut source, _) = tuple((spaces, char('{')))(source)?;

    if let Ok((source, _)) = tuple((spaces, char('}')))(source) {
        return Ok((source, ()));
    }

    // Parse each block
    loop {
        // Parse label if any
        let (mut source_, label) = opt(preceded(
            spaces,
            terminated(name::parse, preceded(spaces, char(':'))),
        ))(source)?;
        let label = label.unwrap_or(name::Name::Number(num_args));

        let block = ctx.get_or_create_named_block(label);

        ctx.layout.append_block(block);
        ctx.cur_block = block;

        while let Ok((source__, inst)) = instruction::parse(source_, ctx) {
            ctx.layout.append_inst(inst, ctx.cur_block);
            source_ = source__
        }

        if let Ok((source, _)) = tuple((spaces, char('}')))(source_) {
            ctx.set_blocks_info();
            return Ok((source, ()));
        }

        source = source_
    }
}

pub fn parse<'a>(
    source: &'a str,
    types: Types,
) -> IResult<&'a str, Function, VerboseError<&'a str>> {
    let (source, define_or_declare) =
        preceded(spaces, alt((tag("define"), tag("declare"))))(source)?;
    let is_prototype = define_or_declare == "declare";
    let (source, preemption_specifier) =
        opt(preceded(spaces, preemption_specifier::parse))(source)?;
    let (source, result_ty) = types::parse(source, &types)?;
    let (source, (_, _, _, name)) = tuple((spaces, char('@'), spaces, alphanumeric1))(source)?;
    let (source, params) = parse_argument_list(source, &types)?;
    let (mut source, fn_attrs) = parse_func_attrs(source)?;

    let mut data = Data::new();
    let mut layout = Layout::new();
    let mut name_to_value = FxHashMap::default();
    let mut name_to_block = FxHashMap::default();
    let dummy_block = data.create_block();

    for (i, param) in params.iter().enumerate() {
        let arg = data.create_value(Value::Argument(i));
        name_to_value.insert(param.name.clone(), arg);
    }

    if !is_prototype {
        source = parse_body(
            source,
            &mut ParserContext {
                types: &types,
                data: &mut data,
                layout: &mut layout,
                name_to_value: &mut name_to_value,
                name_to_block: &mut name_to_block,
                cur_block: dummy_block,
            },
            params.len(),
        )?
        .0;
    }

    Ok((
        source,
        Function {
            name: name.to_string(),
            is_var_arg: false,
            result_ty,
            preemption_specifier: preemption_specifier
                .unwrap_or(preemption_specifier::PreemptionSpecifier::DsoPreemptable),
            attributes: fn_attrs,
            params,
            data,
            layout,
            types,
            is_prototype,
        },
    ))
}

impl<'a> ParserContext<'a> {
    pub fn get_or_create_named_value(&mut self, name: name::Name) -> ValueId {
        if let Some(value) = self.name_to_value.get(&name) {
            return *value;
        }
        let dummy = self
            .data
            .create_inst(Opcode::Invalid.with_block(self.cur_block));
        let id = self.data.create_value(Value::Instruction(dummy));
        self.name_to_value.insert(name, id);
        id
    }

    pub fn get_or_create_named_block(&mut self, name: name::Name) -> BasicBlockId {
        if let Some(block) = self.name_to_block.get(&name) {
            return *block;
        }
        let block = self.data.create_block();
        self.data.block_ref_mut(block).name = Some(name.clone());
        self.name_to_block.insert(name, block);
        block
    }

    fn set_blocks_info(&mut self) {
        for block_id in self.layout.block_iter() {
            let maybe_br = self.layout.basic_blocks[&block_id].last_inst;
            let maybe_br = if let Some(maybe_br) = maybe_br {
                maybe_br
            } else {
                continue;
            };
            let maybe_br = &self.data.instructions[maybe_br];
            if !maybe_br.opcode.is_terminator() {
                continue;
            }
            let br = maybe_br;
            match br.operand {
                Operand::Br { block } => {
                    self.data.basic_blocks[br.parent].succs.insert(block);
                    self.data.basic_blocks[block].preds.insert(br.parent);
                }
                Operand::CondBr { blocks, .. } => {
                    for &block in blocks.iter() {
                        self.data.basic_blocks[br.parent].succs.insert(block);
                        self.data.basic_blocks[block].preds.insert(br.parent);
                    }
                }
                _ => continue,
            }
        }
    }
}

#[test]
fn test_parse_function1() {
    let types = Types::new();
    let result = parse(
        r#"
        define dso_local i32 @main(i32 %0, i32 %1) {
            ret i32 0
        }
        "#,
        types,
    );
    assert!(result.is_ok());
    let result = result.unwrap().1;
    assert_eq!(result.name, "main");
    assert_eq!(
        result.preemption_specifier,
        preemption_specifier::PreemptionSpecifier::DsoLocal
    );
    println!("{:?}", result);
}

#[test]
fn test_parse_function2() {
    let types = Types::new();
    let result = parse(
        r#"
        define dso_local i32 @main() #0 noinline {
        entry:
            %1 = alloca i32, align 4
            store i32 1, i32* %1
            ret i32 0
        }
        "#,
        types,
    );
    assert!(result.is_ok());
    let result = result.unwrap().1;
    assert_eq!(result.name, "main");
    assert_eq!(
        result.preemption_specifier,
        preemption_specifier::PreemptionSpecifier::DsoLocal
    );
    println!("{:?}", result);
}
