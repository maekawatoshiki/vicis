pub mod analysis;

use rustc_hash::FxHashMap;
use std::any::{Any, TypeId};

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
    results: FxHashMap<TypeId, Box<dyn Any>>,
}

impl<T> PassManager<T> {
    pub fn new() -> Self {
        Self {
            passes: vec![],
            results: FxHashMap::default(),
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
            self.results.insert((*result).type_id(), result);
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
            self.results.insert((*result).type_id(), result);
        }
    }

    pub fn get_result<P: 'static>(&self) -> Option<&P> {
        self.results
            .get(&TypeId::of::<P>())
            .map_or(None, |result| result.downcast_ref())
    }
}

use crate::ir::{function::Function, module::Module};

impl PassManager<Function> {
    pub fn run_analyses_on_module(&mut self, module: &Module) {
        for (_, func) in &module.functions {
            self.run_analyses_on(func)
        }
    }

    pub fn run_on_module(&mut self, module: &mut Module) {
        for (_, func) in &mut module.functions {
            self.run_on(func)
        }
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

        pm.run_analyses_on_module(&module);

        assert_eq!(
            pm.get_result::<TestFunctionAnalysisResult>().unwrap().0,
            "main"
        );
    }

    #[test]
    fn analysis_transform() {
        let mut module = test_module();

        let mut pm = PassManager::new();
        pm.add_analysis(TestFunctionAnalysisPass {});
        pm.add_transform(TestFunctionTransformPass {});

        pm.run_on_module(&mut module);

        assert_eq!(
            pm.get_result::<TestFunctionAnalysisResult>().unwrap().0,
            "main"
        );
    }
}
