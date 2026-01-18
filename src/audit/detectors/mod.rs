//! Architecture detectors for identifying gaps and improvements.
//!
//! This module contains detectors that analyze the codebase architecture
//! and identify areas where improvements can be made.

pub mod architecture_gaps;
pub mod opportunities;
pub mod tech_debt;

pub use architecture_gaps::{
    ArchitectureGap, ArchitectureGapType, ArchitectureGapsAnalysis, ArchitectureGapsDetector,
};
pub use opportunities::{
    OpportunityAnalysis, OpportunityContext, OpportunityDetector, OpportunityPattern,
    OpportunityType,
};
pub use tech_debt::{TechDebtAnalysis, TechDebtDetector, TechDebtItem, TechDebtType};
