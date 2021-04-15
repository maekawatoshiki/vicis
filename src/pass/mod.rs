pub mod analysis;

use std::any::Any;

pub trait AnalysisPass<T> {
    fn run_on(&self, _: &T, _: &mut Box<dyn Any>) {}
}

pub trait TransformPass<T> {
    fn run_on(&self, _: &mut T, _: &mut Box<dyn Any>) {}
}

pub enum Pass<T> {
    Analysis(Box<dyn AnalysisPass<T>>),
    Transform(Box<dyn TransformPass<T>>),
}

pub struct PassManager<T> {
    passes: Vec<Pass<T>>,
    results: Vec<Box<dyn Any>>,
}

impl<T> PassManager<T> {
    pub fn new() -> Self {
        Self {
            passes: vec![],
            results: vec![],
        }
    }

    pub fn add(&mut self, pass: Pass<T>) {
        self.passes.push(pass)
    }

    pub fn add_analysis<P: 'static + AnalysisPass<T>>(&mut self, pass: P) {
        self.passes.push(Pass::Analysis(Box::new(pass)))
    }

    pub fn add_transform<P: 'static + TransformPass<T>>(&mut self, pass: P) {
        self.passes.push(Pass::Transform(Box::new(pass)))
    }

    pub fn run_on(&mut self, target: &mut T) {
        self.results.clear();

        for pass in &self.passes {
            let mut result: Box<dyn Any> = Box::new(());
            match pass {
                Pass::Analysis(analysis) => analysis.run_on(target, &mut result),
                Pass::Transform(transform) => transform.run_on(target, &mut result),
            }
            self.results.push(result)
        }
    }

    pub fn run_analyses_on(&mut self, target: &T) {
        self.results.clear();

        for pass in &self.passes {
            let mut result: Box<dyn Any> = Box::new(());
            match pass {
                Pass::Analysis(analysis) => analysis.run_on(target, &mut result),
                Pass::Transform(_) => {}
            }
            self.results.push(result)
        }
    }

    pub fn find_result<P: 'static>(&self) -> Option<&P> {
        self.results.iter().find_map(|result| result.downcast_ref())
    }
}

impl<T> Pass<T> {
    pub fn analysis<P: 'static + AnalysisPass<T>>(pass: P) -> Self {
        Self::Analysis(Box::new(pass))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ir::{
        function::Function,
        module::{parse_assembly, Module},
    };

    pub struct TestFunctionAnalysisPass {}
    pub struct TestFunctionAnalysisResult(String);

    impl AnalysisPass<Function> for TestFunctionAnalysisPass {
        fn run_on(&self, func: &Function, result: &mut Box<dyn Any>) {
            *result = Box::new(TestFunctionAnalysisResult(func.name.to_owned()));
        }
    }

    pub struct TestFunctionTransformPass {}

    impl TransformPass<Function> for TestFunctionTransformPass {
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

        let mut pm = PassManager::new();
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

        let mut pm = PassManager::new();
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
