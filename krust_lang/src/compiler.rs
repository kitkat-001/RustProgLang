//! The module for compiling source code into byte code.

use crate::{lexer, log, parser};
use lexer::{Token, TokenType};
use log::{is_error, ErrorType, Log, LogType};
use parser::{Expression, ParserOutput, Type};

use num_derive::FromPrimitive;

/// The `OpCode` used in the bytecode.
#[derive(FromPrimitive, Clone, Copy)]
pub enum OpCode {
    // Stack operators
    PushInt,
    PushByte,
    PopInt,
    PopByte,
    PrintInt,
    PrintBool,

    // Arithmetic operators
    MinusInt,
    AddInt,
    SubtractInt,
    MultiplyInt,
    DivideInt,
    ModuloInt,

    // Comparison operators.
    LessInt,
    LessEqualInt,
    GreaterInt,
    GreaterEqualInt,

    // Boolean operators
    Not,

    // Bitwise operators
    ComplementInt,
    AndInt,
    AndByte,
    XorInt,
    XorByte,
    OrInt,
    OrByte,

    // Shifts
    LeftShiftInt,
    RightShiftInt,

    // Equality operators
    EqualityInt,
    EqualityByte,
    InequalityInt,
    InequalityByte,
}

/// The output given by the compiler.
pub struct CompilerOutput {
    pub file_text: String,
    pub bytecode: Option<Vec<u8>>,
    pub logs: Vec<Log>,
}

/// Compiles to bytecode.
#[must_use]
#[allow(clippy::missing_panics_doc)] // Should never actually panic.
pub fn compile(parser_output: ParserOutput, cli_args: [u8; 2]) -> CompilerOutput {
    let mut bytecode: Option<Vec<u8>> = None;
    let mut logs: Vec<Log> = parser_output.logs.clone();

    if !is_error(&logs) {
        let mut byte_list: Vec<u8> = cli_args.to_vec();
        let expr_type: Type = parser_output
            .expr
            .get_type()
            .expect("any \"None\" should have a parsing error");
        byte_list.append(&mut generate_bytecode(&parser_output.expr, cli_args[0]));
        if expr_type != Type::Unit {
            byte_list.push(match expr_type {
                Type::Int => OpCode::PrintInt,
                Type::Bool => OpCode::PrintBool,
                Type::Unit => panic!("Should have been caught by above if statement."),
            } as u8);
        }
        if u32::from(cli_args[0]) * 8 < usize::BITS && byte_list.len() >= 1 << (cli_args[0] * 8) {
            logs.push(Log {
                log_type: LogType::Error(ErrorType::ExcessiveBytecode),
                line_and_col: None,
            });
        } else {
            bytecode = Some(byte_list);
        }
    }

    CompilerOutput {
        file_text: parser_output.file_text,
        bytecode,
        logs,
    }
}

fn generate_bytecode(expr: &Expression, ptr_size: u8) -> Vec<u8> {
    let mut bytecode: Vec<u8> = Vec::new();
    match expr {
        Expression::Binary {
            left,
            op,
            right,
            expr_type,
        } => {
            handle_binary(
                &mut bytecode,
                ptr_size,
                left,
                *op,
                right,
                expr_type.expect("any \"None\" should have a parsing error"),
            );
        }
        Expression::ExpressionList { list } => {
            for expr in list {
                bytecode.append(&mut generate_bytecode(expr, ptr_size));
            }
        }
        Expression::Grouping { expr: child, .. } => {
            bytecode.append(&mut generate_bytecode(child, ptr_size));
        }
        Expression::Literal { token, .. } => {
            handle_literal(&mut bytecode, *token);
        }
        Expression::Statement { expr } => {
            bytecode.append(&mut generate_bytecode(expr, ptr_size));
            if expr.get_type() != Some(Type::Unit) {
                bytecode.push(match expr.get_type() {
                    Some(Type::Int) => OpCode::PopInt,
                    Some(Type::Bool) => OpCode::PopByte,
                    _ => panic!("This type is invalid."),
                } as u8);
            }
        }
        Expression::Unary {
            op, expr: child, ..
        } => {
            bytecode.append(&mut generate_bytecode(child, ptr_size));
            bytecode.push(match op.token_type {
                TokenType::Minus => OpCode::MinusInt,
                TokenType::Tilde => OpCode::ComplementInt,
                TokenType::ExclamationMark => OpCode::Not,
                _ => panic!("all unary operators should have been accounted for"),
            } as u8);
        }
        Expression::Unit => {}
        _ => panic!("all expression types should have been accounted for"),
    }
    bytecode
}

// Handles binary expressions.
fn handle_binary(
    bytecode: &mut Vec<u8>,
    ptr_size: u8,
    left: &Expression,
    op: Token,
    right: &Expression,
    expr_type: Type,
) {
    bytecode.append(&mut generate_bytecode(left, ptr_size));
    bytecode.append(&mut generate_bytecode(right, ptr_size));
    match op.token_type {
        TokenType::Plus => {
            bytecode.push(OpCode::AddInt as u8);
        }
        TokenType::Minus => {
            bytecode.push(OpCode::SubtractInt as u8);
        }
        TokenType::Star => {
            bytecode.push(OpCode::MultiplyInt as u8);
        }
        TokenType::Slash => {
            bytecode.push(OpCode::DivideInt as u8);
            bytecode.append(&mut usize_to_ptr_size(op.line, ptr_size));
            bytecode.append(&mut usize_to_ptr_size(op.col, ptr_size));
        }
        TokenType::Percent => {
            bytecode.push(OpCode::ModuloInt as u8);
            bytecode.append(&mut usize_to_ptr_size(op.line, ptr_size));
            bytecode.append(&mut usize_to_ptr_size(op.col, ptr_size));
        }

        TokenType::Less => {
            bytecode.push(OpCode::LessInt as u8);
        }
        TokenType::LessEqual => {
            bytecode.push(OpCode::LessEqualInt as u8);
        }
        TokenType::Greater => {
            bytecode.push(OpCode::GreaterInt as u8);
        }
        TokenType::GreaterEqual => {
            bytecode.push(OpCode::GreaterEqualInt as u8);
        }

        TokenType::Ampersand => {
            bytecode.push(match expr_type {
                Type::Int => OpCode::AndInt,
                Type::Bool => OpCode::AndByte,
                Type::Unit => panic!("Invalid type for this operation"),
            } as u8);
        }
        TokenType::Caret => {
            bytecode.push(match expr_type {
                Type::Int => OpCode::XorInt,
                Type::Bool => OpCode::XorByte,
                Type::Unit => panic!("Invalid type for this operation"),
            } as u8);
        }
        TokenType::Bar => {
            bytecode.push(match expr_type {
                Type::Int => OpCode::OrInt,
                Type::Bool => OpCode::OrByte,
                Type::Unit => panic!("Invalid type for this operation"),
            } as u8);
        }

        TokenType::LeftShift => {
            bytecode.push(OpCode::LeftShiftInt as u8);
        }
        TokenType::RightShift => {
            bytecode.push(OpCode::RightShiftInt as u8);
        }

        TokenType::Equality => {
            bytecode.push(match &left.get_type() {
                Some(Type::Int) => OpCode::EqualityInt,
                Some(Type::Bool) => OpCode::EqualityByte,
                _ => panic!("No other type should be possible."),
            } as u8);
        }
        TokenType::Inequality => {
            bytecode.push(match &left.get_type() {
                Some(Type::Int) => OpCode::InequalityInt,
                Some(Type::Bool) => OpCode::InequalityByte,
                _ => panic!("No other type should be possible."),
            } as u8);
        }

        _ => {
            panic!("invalid token found at head of binary expression.")
        }
    }
}

// Handles literal expressions/tokens.
fn handle_literal(bytecode: &mut Vec<u8>, token: Token) {
    match token.token_type {
        TokenType::IntLiteral(value) => {
            bytecode.push(OpCode::PushInt as u8);
            bytecode.append(&mut value.to_le_bytes().to_vec());
        }
        TokenType::True => {
            bytecode.push(OpCode::PushByte as u8);
            bytecode.push(1u8);
        }
        TokenType::False => {
            bytecode.push(OpCode::PushByte as u8);
            bytecode.push(0u8);
        }
        _ => panic!("all literals should have been accounted for"),
    }
}

// Converts a usize value to a list of bytes with a length of ptr_size.
fn usize_to_ptr_size(value: usize, ptr_size: u8) -> Vec<u8> {
    let usize_size_bytes: u32 = usize::BITS / 8;
    let bytes: [u8; 8] = value.to_le_bytes();
    let bytes: &[u8] = &bytes[..u32::min(u32::from(ptr_size), usize_size_bytes) as usize];
    let mut bytes: Vec<u8> = bytes.to_vec();
    while bytes.len() < ptr_size as usize {
        bytes.push(0);
    }
    bytes
}
