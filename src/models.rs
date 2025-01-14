use std::fmt::{
    self,
    Debug,
    Display,
    Formatter,
};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Model {
    OpenAIGPT4o,
    OpenAIGPT4oMini,
    OpenAIGPTo1Preview,
    OpenAIGPTo1Mini,
    OpenAIGPTo1,
    Invalid,
}

impl Default for Model {
    fn default() -> Self {
        Self::OpenAIGPT4o
    }
}

impl From<&str> for Model {
    fn from(s: &str) -> Self {
        match s {
            "4o" => Self::OpenAIGPT4o,
            "4o-mini" | "4om" => Self::OpenAIGPT4oMini,
            "o1-preview" | "o1p" => Self::OpenAIGPTo1Preview,
            "o1-mini" | "o1m" => Self::OpenAIGPTo1Mini,
            "o1" => Self::OpenAIGPTo1,
            _ => Self::Invalid,
        }
    }
}

impl Display for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenAIGPT4o => write!(f, "gpt-4o"),
            Self::OpenAIGPT4oMini => write!(f, "gpt-4o-mini"),
            Self::OpenAIGPTo1Preview => write!(f, "o1-preview"),
            Self::OpenAIGPTo1Mini => write!(f, "o1-mini"),
            Self::OpenAIGPTo1 => write!(f, "o1"),
            Self::Invalid => panic!("Invalid model"),
        }
    }
}
