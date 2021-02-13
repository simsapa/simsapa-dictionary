use std::{error, fmt};

pub enum ToolError {
    Exit(String),
}

impl fmt::Display for ToolError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolError::Exit(s) => {
                write!(fmt, "{}", s)
            }
        }
    }
}

impl fmt::Debug for ToolError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind: &'static str = match *self {
            ToolError::Exit(_) => "ExitError",
        };

        write!(fmt, "{}:\n{}", kind, self.to_string())
    }
}

impl error::Error for ToolError {
    fn description(&self) -> &str {
        match *self {
            ToolError::Exit(ref s) => s.trim(),
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}
