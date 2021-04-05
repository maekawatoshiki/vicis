pub mod analysis;

use crate::ir::function::Function;
use std::any::Any;

pub trait FunctionAnalysisPass {
    fn run_on(&self, _: &Function, _: &mut Box<dyn Any>) {}
}

pub trait FunctionTransformPass {
    fn run_on(&self, _: &mut Function, _: &mut Box<dyn Any>) {}
}

pub enum FunctionPass {
    Analysis(Box<dyn FunctionAnalysisPass>),
    Transform(Box<dyn FunctionTransformPass>),
}

pub struct FunctionPassManager {
    passes: Vec<FunctionPass>,
    results: Vec<Box<dyn Any>>,
}

impl FunctionPassManager {
    pub fn new() -> Self {
        Self {
            passes: vec![],
            results: vec![],
        }
    }

    pub fn add(&mut self, pass: FunctionPass) {
        self.passes.push(pass)
    }

    pub fn add_analysis<T: 'static + FunctionAnalysisPass>(&mut self, pass: T) {
        self.passes.push(FunctionPass::Analysis(Box::new(pass)))
    }

    pub fn add_transform<T: 'static + FunctionTransformPass>(&mut self, pass: T) {
        self.passes.push(FunctionPass::Transform(Box::new(pass)))
    }

    pub fn run_on(&mut self, func: &mut Function) {
        self.results.clear();

        for pass in &self.passes {
            let mut result: Box<dyn Any> = Box::new(());
            match pass {
                FunctionPass::Analysis(analysis) => analysis.run_on(func, &mut result),
                FunctionPass::Transform(transform) => transform.run_on(func, &mut result),
            }
            self.results.push(result)
        }
    }

    pub fn run_analyses_on(&mut self, func: &Function) {
        self.results.clear();

        for pass in &self.passes {
            let mut result: Box<dyn Any> = Box::new(());
            match pass {
                FunctionPass::Analysis(analysis) => analysis.run_on(func, &mut result),
                FunctionPass::Transform(_) => {}
            }
            self.results.push(result)
        }
    }

    pub fn find_result<T: 'static>(&self) -> Option<&T> {
        self.results.iter().find_map(|result| result.downcast_ref())
    }
}

impl FunctionPass {
    pub fn analysis<T: 'static + FunctionAnalysisPass>(pass: T) -> Self {
        Self::Analysis(Box::new(pass))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ir::module::{parse_assembly, Module};

    pub struct TestFunctionAnalysisPass {}
    pub struct TestFunctionAnalysisResult(String);

    impl FunctionAnalysisPass for TestFunctionAnalysisPass {
        fn run_on(&self, func: &Function, result: &mut Box<dyn Any>) {
            *result = Box::new(TestFunctionAnalysisResult(func.name.to_owned()));
        }
    }

    pub struct TestFunctionTransformPass {}

    impl FunctionTransformPass for TestFunctionTransformPass {
        fn run_on(&self, _func: &mut Function, _result: &mut Box<dyn Any>) {}
    }

    fn test_module() -> Module {
        parse_assembly(
            r#"
source_filename = "sample"                                                                            
target datalayout = "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"       
target triple = "x86_64-pc-linux-gnu"                                                              
         
define dso_local i32 @main() {                                            
  ret i32 0
}                                                                            
        "#,
        ).expect("failed to parse IR")
    }

    #[test]
    fn analysis() {
        let module = test_module();

        let mut pm = FunctionPassManager::new();
        pm.add_analysis(TestFunctionAnalysisPass {});

        for (_, func) in &module.functions {
            pm.run_analyses_on(func);
        }

        assert_eq!(
            pm.find_result::<TestFunctionAnalysisResult>().unwrap().0,
            "main"
        );
    }

    #[test]
    fn analysis_transform() {
        let mut module = test_module();

        let mut pm = FunctionPassManager::new();
        pm.add_analysis(TestFunctionAnalysisPass {});
        pm.add_transform(TestFunctionTransformPass {});

        for (_, func) in &mut module.functions {
            pm.run_on(func);
        }

        assert_eq!(
            pm.find_result::<TestFunctionAnalysisResult>().unwrap().0,
            "main"
        );
    }
}
