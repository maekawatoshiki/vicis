use vicis::{
    codegen::{isa::x86_64::X86_64, lower::convert_module},
    // exec::{generic_value::GenericValue, interpreter::Interpreter},
    ir::module,
};

#[test]
fn compile_tests() {
    use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
    use std::fs;

    let files_count = fs::read_dir("./tests/codegen")
        .expect("Failed to open file")
        .count() as u64
        / 2;
    let paths = fs::read_dir("./tests/codegen").unwrap();
    let pb = ProgressBar::with_draw_target(files_count, ProgressDrawTarget::stdout());
    pb.set_style(ProgressStyle::default_bar().template("{bar:60} {pos:>4}/{len:>4} {msg}"));

    for path in paths {
        let input = path.as_ref().unwrap().path().to_str().unwrap().to_string();
        println!(">> {}", input);
        if !input.ends_with(".ll") {
            continue;
        }
        // if input.contains("ary3") {
        //     continue;
        // }
        let output = format!("{}.s", input.trim_end_matches(".ll"));
        pb.set_message(input.as_str());

        let input_body = &fs::read_to_string(&input).unwrap();
        let output_body = &fs::read_to_string(output).unwrap();

        let module = module::parse_assembly(input_body).unwrap();
        let mach_module = convert_module(X86_64, module);

        println!("{}", mach_module);

        assert_eq!(
            &format!("{}", mach_module),
            output_body,
            "Failed at {}",
            input
        );

        pb.inc(1);
    }

    pb.finish();
}
