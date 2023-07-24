//! The module for compiling source code into byte code.

use crate::{lexer, parser};
use lexer::TokenType;
use parser::{Expression, ParserOutput};

use num_derive::FromPrimitive;

/// The opcodes used in the bytecode.
#[derive(FromPrimitive)]
pub enum OpCodes
{
    PushInt,
    PopInt,

    MinusInt,

    AddInt,
    SubtractInt,
    MultiplyInt,
    DivideInt,
}

/// The output given by the compiler.
pub struct CompilerOutput
{
    pub file_text: String,
    pub bytecode: Option<Vec<u8>>,
    pub errors: Vec<String>,
}

/// Compiles to bytecode.
pub fn compile(parser_output: ParserOutput, cli_args: [u8; 1]) -> CompilerOutput
{
    let mut bytecode: Option<Vec<u8>> = None;
    let mut errors: Vec<String> = parser_output.errors.clone();

    if parser_output.can_compile
    {
        let mut byte_list: Vec<u8> = cli_args.to_vec();
        byte_list.append(&mut generate_bytecode(parser_output.expr, cli_args[0]));
        byte_list.push(OpCodes::PopInt as u8);
        if u32::from(cli_args[0]) * 8 < usize::BITS && byte_list.len() >= 1 << (cli_args[0] * 8)
        {
            errors.push("error: could not compile as bytecode was too large.".to_string());
        }
        else
        {
            bytecode = Some(byte_list);
        }
    }

    CompilerOutput { file_text: parser_output.file_text, bytecode, errors }
}

fn generate_bytecode(expr: Expression, ptr_size: u8) -> Vec<u8>
{
    let mut bytecode: Vec<u8> = Vec::new();
    match expr
    {
        Expression::Binary { left, op, right } =>
        {
            bytecode.append(&mut generate_bytecode(*left, ptr_size));
            bytecode.append(&mut generate_bytecode(*right, ptr_size));
            match op.token_type
            {
                TokenType::Plus => { bytecode.push(OpCodes::AddInt as u8); },
                TokenType::Minus => { bytecode.push(OpCodes::SubtractInt as u8); },
                TokenType::Star => { bytecode.push(OpCodes::MultiplyInt as u8); },
                TokenType::Slash => 
                {
                     bytecode.push(OpCodes::DivideInt as u8); 
                     bytecode.append(&mut usize_to_ptr_size(op.line, ptr_size));
                     bytecode.append(&mut usize_to_ptr_size(op.col, ptr_size));
                },
                _ => { panic!("invalid token found at head of binary expression.")}
            }
        }
        Expression::Grouping { expr: child } =>
        {
            bytecode.append(&mut generate_bytecode(*child, ptr_size));
        }
        Expression::Literal { token } =>
        {
            if let TokenType::IntLiteral(value) = token.token_type
            {
                bytecode.push(OpCodes::PushInt as u8);
                bytecode.append(&mut value.to_le_bytes().to_vec());
            }
            else 
            {
                panic!("all literals should have been accounted for");
            }
        }
        Expression::Unary { op, expr: child } =>
        {
            bytecode.append(&mut generate_bytecode(*child, ptr_size));
            if op.token_type == TokenType::Minus
            {
                bytecode.push(OpCodes::MinusInt as u8);
            }
        }
        _ => panic!("all expression types should have been accounted for"),
    }
    bytecode
}

fn usize_to_ptr_size(value: usize, ptr_size: u8) -> Vec<u8>
{
    let usize_size_bytes: u32 = usize::BITS / 8;
    let bytes = value.to_le_bytes();
    let bytes: &[u8] = &bytes[..u32::min(ptr_size as u32, usize_size_bytes) as usize];
    let mut bytes: Vec<u8> = bytes.to_vec();
    while bytes.len() < ptr_size as usize
    {
        bytes.push(0);
    } 
    bytes
}