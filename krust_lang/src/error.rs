//! The module for error handling.

use std::fmt::{Display, Formatter, Result};

/// An enum representing anything that can be logged.
#[derive(Clone)]
pub enum LogType
{
    Warning(WarningType),
    Error(ErrorType)
}

#[derive(Clone)]
pub enum WarningType
{
    CLIArgRoundedDownU16(String, u16),
    CLITargetLargerThanMachine(u16),
}

/// An enum representing any possible error.
#[derive(Clone)]
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
    CLIUnrecognizedArg(String),
    CLICantOpenFile(String),
    CLINoFile,
    CLIFileToBig(usize),

    UnrecognizedToken(String),
    UnrepresentableIntegerLiteral(String),
    
    ExpectedEOF,
    UnexpectedEOF,
    UnexpectedToken,
    ExpectedExpressionInParens,
    ExpectedCloseParen,
    UnnegatedMinimumIntegerLiteral,

    ExcessiveBytecode,

    DivideByZero,
}
/// Represents all possible errors as well as helpful debug information when relevant.
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
                return write!(f, "fatal error; program terminated");
            }
        }

        let log_type: String = match self.log_type.clone() {
            LogType::Warning(_) => "warning".to_string(),
            LogType::Error(_) => "error".to_string(),
        };

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
                ErrorType::CLICantReadArgs => "could not read command line arguments".to_string(),
                ErrorType::CLINoArgs => "no command line arguments.".to_string(),
                ErrorType::CLIRequiresArg(arg) 
                    => format!("compiler flag \"{arg}\" requires an argument."),
                ErrorType::CLIRequiresNumArg(arg) 
                    => format!("compiler flag \"{arg}\" requires a numerical argument."),
                ErrorType::CLIRequiresNumArgLessThanU16(arg, bound) 
                    => format!("compiler flag \"{arg}\" requires an argument less than {bound}."),
                ErrorType::CLIRequiresNumArgAtLeastU16(arg, bound) 
                    => format!("compiler flag \"{arg}\" requires an argument that's at least {bound}."),
                ErrorType::CLIUnrecognizedArg(arg)
                    => format!("unrecognized argument \"{arg}\"."),
                ErrorType::CLICantOpenFile(path)
                    => format!("could not open file \"{path}\"."),
                ErrorType::CLINoFile => "no source file entered".to_string(),
                ErrorType::CLIFileToBig(ptr_size) 
                    => format!("error: the file is too big to compile for a {ptr_size}-bit machine"),

                ErrorType::UnrecognizedToken(token) 
                    => format!("unrecognized token \"{token}\"."),
                ErrorType::UnrepresentableIntegerLiteral(token) 
                    => format!("int literal \"{token}\" must be at most {}.", 0x_8000_0000u32),

                ErrorType::ExpectedEOF => "expected end of file.".to_string(),
                ErrorType::UnexpectedEOF => "unexpected end of file.".to_string(),
                ErrorType::UnexpectedToken => "unexpected token.".to_string(),
                ErrorType::ExpectedExpressionInParens => "expected expression within parentheses.".to_string(),
                ErrorType::ExpectedCloseParen => "expected \')\' following \'(\'.".to_string(),
                ErrorType::UnnegatedMinimumIntegerLiteral
                    => format!("the int literal {} must be preceded by a unary \'-\' operator.", 0x8000_0000u32),

                ErrorType::ExcessiveBytecode => "could not compile as bytecode was too large.".to_string(),
            
                ErrorType::DivideByZero => "division by zero.".to_string(),
            }},
        }};
        if let None = self.line_and_col
        {
            write!(f, "{log_type}: {message}")
        }
        else 
        {
            write!(f, "{log_type} (line {}:{}): {message}", 
                self.line_and_col.expect("checked by if statement").0, 
                self.line_and_col.expect("checked by if statement").1)    
        }
    }
}