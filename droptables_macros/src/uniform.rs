pub use error::ProbError;
pub use sampler::UniformSampler;
pub use staticdt::StaticDropTable;
pub use walker::WeightedSampler;
pub use uniform::UniformTable;            // optional: stateful uniform table

pub use droptables_macros::WeightedEnum;  // already there
pub use droptables_macros::UniformEnum;   // <-- new: export the derive

use rand::Rng;

