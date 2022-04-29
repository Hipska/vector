use vrl::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct ForEach;

impl Function for ForEach {
    fn identifier(&self) -> &'static str {
        "for_each"
    }

    fn parameters(&self) -> &'static [Parameter] {
        &[Parameter {
            keyword: "value",
            kind: kind::OBJECT | kind::ARRAY,
            required: true,
        }]
    }

    fn examples(&self) -> &'static [Example] {
        &[
            Example {
                title: "iterate object",
                source: r#"count = 0; for_each({ "a": 1, "b": 2 }) -> |_key, value| { count = count + value }; count"#,
                result: Ok("3"),
            },
            Example {
                title: "iterate array",
                source: r#"count = 0; for_each([1,2,3]) -> |index, value| { count = count + index + value }; count"#,
                result: Ok("9"),
            },
        ]
    }

    fn compile(
        &self,
        _state: (&mut state::LocalEnv, &mut state::ExternalEnv),
        _ctx: &mut FunctionCompileContext,
        mut arguments: ArgumentList,
    ) -> Compiled {
        let value = arguments.required("value");
        let closure = arguments.required_closure()?;

        Ok(Box::new(ForEachFn { value, closure }))
    }

    fn closure(&self) -> Option<closure::Definition> {
        use closure::{Definition, Input, Output, Variable, VariableKind};

        Some(Definition {
            inputs: vec![Input {
                parameter_keyword: "value",
                kind: Kind::object(Collection::any()).or_array(Collection::any()),
                variables: vec![
                    Variable {
                        kind: VariableKind::TargetInnerKey,
                    },
                    Variable {
                        kind: VariableKind::TargetInnerValue,
                    },
                ],
                output: Output::Kind(Kind::any()),
                example: Example {
                    title: "iterate array",
                    source: r#"for_each([1, 2]) -> |index, value| { .foo = to_int!(.foo) + index + value }"#,
                    result: Ok("null"),
                },
            }],
            is_iterator: true,
        })
    }

    fn call_by_vm(&self, _ctx: &mut Context, _args: &mut VmArgumentList) -> Result<Value> {
        // TODO: this work will happen in a follow-up PR
        Err("function currently unavailable in VM runtime".into())
    }
}

#[derive(Debug, Clone)]
struct ForEachFn {
    value: Box<dyn Expression>,
    closure: FunctionClosure,
}

impl Expression for ForEachFn {
    fn resolve(&self, ctx: &mut Context) -> Result<Value> {
        let value = self.value.resolve(ctx)?;
        let mut iter = value.into_iter(false);

        for item in iter.by_ref() {
            match item {
                IterItem::KeyValue(key, value) => self.closure.run_key_value(ctx, key, value)?,

                IterItem::IndexValue(index, value) => {
                    self.closure.run_index_value(ctx, index, value)?
                }

                _ => {}
            };
        }

        Ok(Value::Null)
    }

    fn type_def(&self, ctx: (&state::LocalEnv, &state::ExternalEnv)) -> TypeDef {
        let fallible = self.closure.type_def(ctx).is_fallible();

        TypeDef::null().with_fallibility(fallible)
    }
}