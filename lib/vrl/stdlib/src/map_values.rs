use vrl::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct MapValues;

impl Function for MapValues {
    fn identifier(&self) -> &'static str {
        "map_values"
    }

    fn parameters(&self) -> &'static [Parameter] {
        &[
            Parameter {
                keyword: "value",
                kind: kind::OBJECT | kind::ARRAY,
                required: true,
            },
            Parameter {
                keyword: "recursive",
                kind: kind::BOOLEAN,
                required: false,
            },
        ]
    }

    fn examples(&self) -> &'static [Example] {
        &[
            Example {
                title: "map object values",
                source: r#"map_values({ "a": 1, "b": 2 }) -> |value| { value = int!(value) + 1 }"#,
                result: Ok(r#"{ "a": 2, "b": 3 }"#),
            },
            Example {
                title: "recursively map object values",
                source: r#"map_values({ "a": 1, "b": [{ "c": 2 }, { "d": 3 }], "e": { "f": 4 } }, recursive: true) -> |value| { value = if is_integer(value) { int!(value) + 1 } else { value } }"#,
                result: Ok(r#"{ "a": 2, "b": [{ "c": 3 }, { "d": 4 }], "e": { "f": 5 } }"#),
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
        let recursive = arguments.optional("recursive");
        let closure = arguments.required_closure()?;

        Ok(Box::new(MapValuesFn {
            value,
            closure,
            recursive,
        }))
    }

    fn closure(&self) -> Option<closure::Definition> {
        let input = closure::Input {
            parameter_keyword: "value",
            kind: Kind::object(Collection::any()).or_array(Collection::any()),
            variables: vec![closure::Variable { kind: Kind::any() }],
            output: closure::Output::Kind(Kind::any()),
            example: Example {
                title: "map object values",
                source: r#"map_values({ "one" : "one", "two": "two" }) -> |value| { upcase!(value) }"#,
                result: Ok(r#"{ "one": "ONE", "two": "TWO" }"#),
            },
        };

        Some(closure::Definition {
            inputs: vec![input],
        })
    }

    fn call_by_vm(&self, _ctx: &mut Context, _args: &mut VmArgumentList) -> Result<Value> {
        todo!()
    }
}

#[derive(Debug, Clone)]
struct MapValuesFn {
    value: Box<dyn Expression>,
    recursive: Option<Box<dyn Expression>>,
    closure: FunctionClosure,
}

impl Expression for MapValuesFn {
    fn resolve(&self, ctx: &mut Context) -> Result<Value> {
        let recursive = match &self.recursive {
            None => false,
            Some(expr) => expr.resolve(ctx)?.try_boolean()?,
        };

        let value = self.value.resolve(ctx)?;
        let mut iter = value.into_iter(recursive);

        for item in iter.by_ref() {
            let value = match item {
                IterItem::Value(value) => value,
                IterItem::KeyValue(_, value) => value,
                IterItem::IndexValue(_, value) => value,
            };

            self.closure.map_value(ctx, value)?;
        }

        Ok(iter.into())
    }

    fn type_def(&self, ctx: (&state::LocalEnv, &state::ExternalEnv)) -> TypeDef {
        let type_def = self.closure.type_def(ctx);

        TypeDef::object(Collection::from_unknown(type_def.kind().clone()))
            .with_fallibility(type_def.is_fallible())
    }
}