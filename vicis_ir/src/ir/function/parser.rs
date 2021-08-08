use super::{
    super::{
        function::{
            basic_block::BasicBlockId,
            data::Data,
            instruction,
            instruction::{Opcode, Operand},
            layout::Layout,
            param_attrs::parser::parse_param_attrs,
            Function, Parameter, PersonalityFunc,
        },
        module::{
            attributes, global_variable, linkage, name, preemption_specifier, unnamed_addr,
            visibility,
        },
        types,
        types::Types,
        util::spaces,
        value::{Value, ValueId},
    },
    instruction::{Br, CondBr},
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char,
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
    let (source, attrs) = parse_param_attrs(source, types)?;
    let (source, name) = opt(preceded(spaces, preceded(char('%'), name::parse)))(source)?;
    Ok((
        source,
        Parameter {
            name: name.unwrap_or(name::Name::Number({
                *index += 1;
                *index
            })),
            ty,
            attrs,
        },
    ))
}

pub fn parse_argument_list<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, (Vec<Parameter>, bool), VerboseError<&'a str>> {
    let (mut source, _) = tuple((spaces, char('(')))(source)?;

    if let Ok((source, _)) = tuple((spaces, char(')')))(source) {
        return Ok((source, (vec![], false)));
    }

    let mut params = vec![];
    let mut is_var_arg = false;
    let mut index = 0;

    loop {
        if let Ok((source_, _)) = tuple((spaces, tag("...")))(source) {
            is_var_arg = true;
            source = source_;
            break;
        }

        let (source_, param) = parse_argument(source, types, &mut index)?;
        source = source_;
        params.push(param);

        if let Ok((source_, _)) = tuple((spaces, char(',')))(source_) {
            source = source_;
            continue;
        }

        break;
    }

    let (source, _) = tuple((spaces, char(')')))(source)?;
    Ok((source, (params, is_var_arg)))
}

pub fn parse_body<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
    num_args: usize,
) -> IResult<&'a str, (), VerboseError<&'a str>> {
    let (source, _) = tuple((spaces, char('{')))(source)?;

    if let Ok((source, _)) = tuple((spaces, char('}')))(source) {
        return Ok((source, ()));
    }

    let (mut source, entry) = opt(preceded(
        spaces,
        terminated(name::parse, preceded(spaces, char(':'))),
    ))(source)?;
    let mut label = entry.unwrap_or(name::Name::Number(num_args));

    // Parse each block
    loop {
        let block = ctx.get_or_create_named_block(label);

        ctx.layout.append_block(block);
        ctx.cur_block = block;

        while let Ok((source_, inst)) = instruction::parse(source, ctx) {
            ctx.layout.append_inst(inst, ctx.cur_block);
            source = source_
        }

        if let Ok((source, _)) = tuple((spaces, char('}')))(source) {
            ctx.set_blocks_info();
            return Ok((source, ()));
        }

        // Parse label
        if let Ok((source_, label_)) =
            preceded(spaces, terminated(name::parse, preceded(spaces, char(':'))))(source)
        {
            label = label_;
            source = source_;
            continue;
        }

        todo!("unsupported syntax at {:?}", source)
    }
}

pub fn parse_personality<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, Option<PersonalityFunc>, VerboseError<&'a str>> {
    if let Ok((source, _)) = preceded(spaces, tag("personality"))(source) {
        let (source, (ty, konst)) = global_variable::parse_global_type_and_const(source, types)?;
        return Ok((source, Some((ty, konst))));
    }

    Ok((source, None))
}

pub fn parse(source: &str, types: Types) -> IResult<&str, Function, VerboseError<&str>> {
    let (source, define_or_declare) =
        preceded(spaces, alt((tag("define"), tag("declare"))))(source)?;
    let is_prototype = define_or_declare == "declare";
    let (source, linkage) = opt(preceded(spaces, linkage::parse))(source)?;
    let (source, preemption_specifier) =
        opt(preceded(spaces, preemption_specifier::parse))(source)?;
    let (source, visibility) = opt(preceded(spaces, visibility::parse))(source)?;
    let (source, ret_attrs) = parse_param_attrs(source, &types)?;
    let (source, result_ty) = types::parse(source, &types)?;
    let (source, (_, _, _, name)) = tuple((spaces, char('@'), spaces, name::parse))(source)?;
    let name = name.to_string().cloned().unwrap();
    let (source, (params, is_var_arg)) = parse_argument_list(source, &types)?;
    let (source, unnamed_addr) = opt(preceded(spaces, unnamed_addr::parse))(source)?;
    let (source, func_attrs) = attributes::parser::parse_attributes(source)?;
    let (mut source, personality) = parse_personality(source, &types)?;

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
            name,
            is_var_arg,
            result_ty,
            linkage: linkage.unwrap_or(linkage::Linkage::External),
            preemption_specifier: preemption_specifier
                .unwrap_or(preemption_specifier::PreemptionSpecifier::DsoPreemptable),
            visibility: visibility.unwrap_or(visibility::Visibility::Default),
            unnamed_addr,
            ret_attrs,
            func_attrs,
            params,
            data,
            layout,
            types,
            // is_prototype,
            personality,
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
            let maybe_br = self.layout.block_node(block_id).last_inst();
            let maybe_br = match maybe_br {
                Some(maybe_br) => *maybe_br,
                None => continue,
            };
            let maybe_br = &self.data.instructions[maybe_br];
            if !maybe_br.opcode.is_terminator() {
                continue;
            }
            let br = maybe_br;
            match br.operand {
                Operand::Br(Br { block }) => {
                    self.data.basic_blocks[br.parent].succs.insert(block);
                    self.data.basic_blocks[block].preds.insert(br.parent);
                }
                Operand::CondBr(CondBr { blocks, .. }) => {
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
