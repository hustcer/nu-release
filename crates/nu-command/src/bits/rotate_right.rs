use super::{get_number_bytes, NumberBytes};
use nu_engine::CallExt;
use nu_protocol::ast::Call;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    Category, Example, PipelineData, ShellError, Signature, Span, Spanned, SyntaxShape, Value,
};
use num_traits::int::PrimInt;
use std::fmt::Display;

#[derive(Clone)]
pub struct SubCommand;

impl Command for SubCommand {
    fn name(&self) -> &str {
        "bits rotate-right"
    }

    fn signature(&self) -> Signature {
        Signature::build("bits rotate-right")
            .required("bits", SyntaxShape::Int, "number of bits to rotate right")
            .switch(
                "signed",
                "always treat input number as a signed number",
                Some('s'),
            )
            .named(
                "number-bytes",
                SyntaxShape::String,
                "the size of number in bytes, it can be 1, 2, 4, 8, auto, default value `auto`",
                Some('n'),
            )
            .category(Category::Bits)
    }

    fn usage(&self) -> &str {
        "Bitwise rotate right for integers"
    }

    fn search_terms(&self) -> Vec<&str> {
        vec!["ror"]
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
        let signed = call.has_flag("signed");
        let number_bytes: Option<Spanned<String>> =
            call.get_flag(engine_state, stack, "number-bytes")?;
        let bytes_len = get_number_bytes(&number_bytes);
        if let NumberBytes::Invalid = bytes_len {
            if let Some(val) = number_bytes {
                return Err(ShellError::UnsupportedInput(
                    "the size of number is invalid".to_string(),
                    val.span,
                ));
            }
        }

        input.map(
            move |value| operate(value, bits, head, signed, bytes_len),
            engine_state.ctrlc.clone(),
        )
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                description: "Rotate right a number with 2 bits",
                example: "17 | bits rotate-right 2",
                result: Some(Value::Int {
                    val: 68,
                    span: Span::test_data(),
                }),
            },
            Example {
                description: "Rotate right a list of numbers",
                example: "[15 33 92] | bits rotate-right 2",
                result: Some(Value::List {
                    vals: vec![
                        Value::test_int(195),
                        Value::test_int(72),
                        Value::test_int(23),
                    ],
                    span: Span::test_data(),
                }),
            },
        ]
    }
}

fn get_rotate_right<T: Display + PrimInt>(val: T, bits: u32, span: Span) -> Value
where
    i64: std::convert::TryFrom<T>,
{
    let rotate_result = i64::try_from(val.rotate_right(bits));
    match rotate_result {
        Ok(val) => Value::Int { val, span },
        Err(_) => Value::Error {
            error: ShellError::GenericError(
                "Rotate right result beyond the range of 64 bit signed number".to_string(),
                format!(
                    "{} of the specified number of bytes rotate right {} bits",
                    val, bits
                ),
                Some(span),
                None,
                Vec::new(),
            ),
        },
    }
}

fn operate(value: Value, bits: usize, head: Span, signed: bool, number_size: NumberBytes) -> Value {
    match value {
        Value::Int { val, span } => {
            use NumberBytes::*;
            let bits = bits as u32;
            if signed || val < 0 {
                match number_size {
                    One => get_rotate_right(val as i8, bits, span),
                    Two => get_rotate_right(val as i16, bits, span),
                    Four => get_rotate_right(val as i32, bits, span),
                    Eight => get_rotate_right(val as i64, bits, span),
                    Auto => {
                        if val <= 0x7F && val >= -(2i64.pow(7)) {
                            get_rotate_right(val as i8, bits, span)
                        } else if val <= 0x7FFF && val >= -(2i64.pow(15)) {
                            get_rotate_right(val as i16, bits, span)
                        } else if val <= 0x7FFFFFFF && val >= -(2i64.pow(31)) {
                            get_rotate_right(val as i32, bits, span)
                        } else {
                            get_rotate_right(val as i64, bits, span)
                        }
                    }
                    // This case shouldn't happen here, as it's handled before
                    Invalid => Value::Int { val, span },
                }
            } else {
                match number_size {
                    One => get_rotate_right(val as u8, bits, span),
                    Two => get_rotate_right(val as u16, bits, span),
                    Four => get_rotate_right(val as u32, bits, span),
                    Eight => get_rotate_right(val as u64, bits, span),
                    Auto => {
                        if val <= 0xFF {
                            get_rotate_right(val as u8, bits, span)
                        } else if val <= 0xFFFF {
                            get_rotate_right(val as u16, bits, span)
                        } else if val <= 0xFFFFFFFF {
                            get_rotate_right(val as u32, bits, span)
                        } else {
                            get_rotate_right(val as u64, bits, span)
                        }
                    }
                    // This case shouldn't happen here, as it's handled before
                    Invalid => Value::Int { val, span },
                }
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
