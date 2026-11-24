/// How a fork's branch relates to its upstream branch.
///
/// Counts are from the fork's point of view: `behind` is the number of upstream commits the fork
/// lacks, and `ahead` is the number of fork commits that upstream lacks.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Divergence {
    /// The fork and upstream point at the same commit.
    Identical,
    /// Upstream has commits the fork lacks, and the fork has nothing extra.
    Behind { by: u32 },
    /// The fork has commits upstream lacks, and upstream has nothing new.
    Ahead { by: u32 },
    /// Both sides have commits the other lacks.
    Diverged { behind: u32, ahead: u32 },
}
