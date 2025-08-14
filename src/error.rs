#[derive(Debug)]
pub enum ProbError {
    Empty,
    Negative { index: usize, value: f32 },
    ZeroSum,
}

impl std::fmt::Display for ProbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProbError::Empty => write!(f, "weights slice is empty"),
            ProbError::Negative { index, value } => {
                write!(
                    f,
                    "weights contain a negative value at index {index}: {value}"
                )
            }
            ProbError::ZeroSum => write!(f, "sum of weights is zero"),
        }
    }
}

impl std::error::Error for ProbError {}
