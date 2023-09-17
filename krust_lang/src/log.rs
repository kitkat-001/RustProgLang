//! The module for debug messages.

use std::fmt::{Display, Formatter, Result};
use colored::{ColoredString, Colorize, control::set_override};

/// An enum representing anything that can be logged.
#[derive(Clone, PartialEq, Eq)]
pub enum LogType
{
    Warning(WarningType),
    Error(ErrorType)
}

/// An enum representing any possible warning.
#[derive(Clone, PartialEq, Eq)]
pub enum WarningType
{
    CLIArgRoundedDownU16(String, u16),
    CLITargetLargerThanMachine(usize),
}

/// An enum representing any possible error.
#[derive(Clone, PartialEq, Eq)]
pub enum ErrorType
{
    FatalError,

    CLIMultipleFiles,
    CLICantReadArgs,
    CLINoArgs,
    CLIRequiresArg(String),
    CLIRequiresNumArg(String),
    CLIRequiresNumArgLessThanU16(String, u16),
    CLIRequiresNumArgAtLeastU16(String, u16),
    CLIRequiresBoolArg(String),
    CLIUnrecognizedArg(String),
    CLICantOpenFile(String),
    CLINoFile,
    CLIFileToBig(usize),

    UnrecognizedToken(String),
    UnrepresentableIntegerLiteral(String),
    InvalidArgsForOperator(String, Vec<String>),
    
    ExpectedEOF,
    UnexpectedEOF,
    UnexpectedToken,
    ExpectedExpressionInParens,
    ExpectedCloseParen,
    UnnegatedMinimumIntegerLiteral,

    ExcessiveBytecode,

    CantCompile,

    CompiledForDifferentTarget(usize),
    DivideByZero,
}

/// Represents all possible errors as well as helpful debug information when relevant.
#[derive(Clone, PartialEq, Eq)]
pub struct Log
{
    pub log_type: LogType,
    pub line_and_col: Option<(usize, usize)>
}

impl Display for Log
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result
    {
        if let LogType::Error(error_type) = self.log_type.clone()
        {
            if let ErrorType::FatalError = error_type
            {
                let error: ColoredString = "fatal error; program terminated".to_string().red().bold();
                return write!(f, "{error}");
            }
        }

        let log_type: ColoredString = match self.log_type.clone() {
            LogType::Warning(_) => "warning".to_string().yellow(),
            LogType::Error(_) => "error".to_string().red(),
        }.bold();

        let mut message_is_bold: bool = true;
        let message: String = { match self.log_type.clone() {
            LogType::Warning(warning_type) => {match warning_type
            {
                WarningType::CLIArgRoundedDownU16(arg, value)
                    => format!("argument of \"{arg}\" will be rounded down to the nearest multiple of {value}."),
                WarningType::CLITargetLargerThanMachine(ptr_size)
                    => format!("warning: this program is being compiled for a {ptr_size}-bit machine, while this is only a {}-bit machine.", 
                    usize::BITS)
            }},
            LogType::Error(error_type) => {match error_type
            {
                ErrorType::FatalError => String::new(), // dealt with above

                ErrorType::CLIMultipleFiles => "command line contains multiple files.".to_string(),
                ErrorType::CLICantReadArgs => "could not read command line arguments.".to_string(),
                ErrorType::CLINoArgs => "no command line arguments.".to_string(),
                ErrorType::CLIRequiresArg(arg) 
                    => format!("compiler flag \"{arg}\" requires an argument."),
                ErrorType::CLIRequiresNumArg(arg) 
                    => format!("compiler flag \"{arg}\" requires a numerical argument."),
                ErrorType::CLIRequiresNumArgLessThanU16(arg, bound) 
                    => format!("compiler flag \"{arg}\" requires an argument less than {bound}."),
                ErrorType::CLIRequiresNumArgAtLeastU16(arg, bound) 
                    => format!("compiler flag \"{arg}\" requires an argument that's at least {bound}."),
                ErrorType::CLIRequiresBoolArg(arg) 
                    => format!("compiler flag \"{arg}\" requires a boolean argument."),
                ErrorType::CLIUnrecognizedArg(arg)
                    => format!("unrecognized argument \"{arg}\"."),
                ErrorType::CLICantOpenFile(path)
                    => format!("could not open file \"{path}\"."),
                ErrorType::CLINoFile => "no source file entered.".to_string(),
                ErrorType::CLIFileToBig(ptr_size) 
                    => format!("the file is too big to compile for a {ptr_size}-bit machine."),

                ErrorType::UnrecognizedToken(token) 
                    => format!("unrecognized token \"{token}\"."),
                ErrorType::UnrepresentableIntegerLiteral(token) 
                    => format!("int literal \"{token}\" must be at most {}.", 0x_8000_0000u32),
                ErrorType::InvalidArgsForOperator(op, types)
                    => format!("the operator \"{op}\" has no definition over the types {}", format_vec_string(types).unwrap_or("".to_string())),

                ErrorType::ExpectedEOF => "expected end of file.".to_string(),
                ErrorType::UnexpectedEOF => "unexpected end of file.".to_string(),
                ErrorType::UnexpectedToken => "unexpected token.".to_string(),
                ErrorType::ExpectedExpressionInParens => "expected expression within parentheses.".to_string(),
                ErrorType::ExpectedCloseParen => "expected \')\' following \'(\'.".to_string(),
                ErrorType::UnnegatedMinimumIntegerLiteral
                    => format!("the int literal {} must be preceded by a unary \'-\' operator.", 0x8000_0000u32),

                ErrorType::ExcessiveBytecode => "could not compile as bytecode was too large.".to_string(),

                ErrorType::CantCompile => {
                    message_is_bold = false;
                    "could not compile due to errors.".to_string()
                }
            
                
                ErrorType::CompiledForDifferentTarget(ptr_size) 
                    => format!("this program was compiled for a {ptr_size}-bit machine, while this is only a {}-bit machine.", usize::BITS),
                ErrorType::DivideByZero => "division by zero.".to_string(),
            }},
        }};
        
        let mut output: String = if self.line_and_col.is_none()
        {
            format!("{log_type}: {message}")
            
        }
        else 
        {
            format!("{log_type} (line {}:{}): {message}", 
                self.line_and_col.expect("checked by if statement").0, 
                self.line_and_col.expect("checked by if statement").1)
        };
        if message_is_bold
        {
            output = output.bold().to_string();
        }
        write!(f, "{output}")
    }
}

/// Returns whether or not a list of logs contains an error.
pub fn is_error(logs: &Vec<Log>) -> bool
{
    for log in logs
    {
        if let LogType::Error(_) = log.log_type
        {
            return true;
        }
    }
    false
}

/// Converts all logs into strings and disables colorizing. Used for testing.
pub fn all_to_string(logs: &Vec<Log>) -> Vec<String>
{
    let mut strings: Vec<String> = Vec::new();
    set_override(false);
    for log in logs
    {
        strings.push(ColoredString::from(format!("{log}").as_str()).clear().to_string());
    }
    strings
}

// Formats a vector of strings into a list with commas and "and".
fn format_vec_string(vec: Vec<String>) -> Option<String>
{
    match vec.len()
    {
        0 => None,
        1 => Some(vec[0].clone()),
        2 => Some({
            let mut value: String = vec[0].clone();
            value.push_str(" and ");
            value.push_str(&vec[1]);
            value
        }),
        _ => Some({
            let mut value: String = vec[0].clone();
            for i in 1..vec.len()-1
            {
                value.push_str(", ");
                value.push_str(&vec[i]);
            }
            value.push_str(", and ");
            value.push_str(&vec[vec.len()-1]);
            value
        })
    }
}