use nu_engine::CallExt;
use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    Category, Example, PipelineData, ShellError, Signature, Span, SyntaxShape, Value,
};

#[derive(Clone)]
pub struct SubCommand;

impl Command for SubCommand {
    fn name(&self) -> &str {
        "bits shift-left"
    }

    fn signature(&self) -> Signature {
        Signature::build("bits shift-left")
            .required("bits", SyntaxShape::Int, "number of bits to shift left")
            .category(Category::Bits)
    }

    fn usage(&self) -> &str {
        "Bitwise shift left for integers"
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["shl"]
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &Call,
        input: PipelineData,
    ) -> Result<nu_protocol::PipelineData, nu_protocol::ShellError> {
        let head = call.head;
        let bits: usize = call.req(engine_state, stack, 0)?;

        input.map(
            move |value| operate(value, bits, head),
            engine_state.ctrlc.clone(),
        )
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Shift left a number with 2 bits",
                example: "2 | bits shift-left 8",
                result: Some(Value::Int {
                    val: 2,
                    span: Span::test_data(),
                }),
            },
            Example {
                description: "Shift left a list of numbers",
                example: "[5 3 2] | bits shift-left 2",
                result: Some(Value::List {
                    vals: vec![Value::test_int(20), Value::test_int(12), Value::test_int(8)],
                    span: Span::test_data(),
                }),
            },
        ]
    }
}

fn operate(value: Value, bits: usize, head: Span) -> Value {
    match value {
        Value::Int { val, span } => {
            let shift_bits = (((bits % 64) + 64) % 64) as u32;
            match val.checked_shl(shift_bits) {
                Some(val) => Value::Int { val, span },
                None => Value::Error {
                    error: ShellError::GenericError(
                        format!("Shift left overflow {} << {}", val, shift_bits),
                        "".to_string(),
                        Some(span),
                        None,
                        Vec::new(),
                    ),
                },
            }
        }
        other => Value::Error {
            error: ShellError::UnsupportedInput(
                format!(
                    "Only integer values are supported, input type: {:?}",
                    other.get_type()
                ),
                other.span().unwrap_or(head),
            ),
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_examples() {
        use crate::test_examples;

        test_examples(SubCommand {})
    }
}
