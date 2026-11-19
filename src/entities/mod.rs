pub mod crumble_block;
pub mod crystal;
pub mod dash_block;
pub mod falling_block;
pub mod finish;
pub mod jump_thru;
pub mod solid;
pub mod spikes;
pub mod spring;
pub mod zipper;

pub use crumble_block::{CrumbleBlock, CrumbleStage};
pub use crystal::Crystal;
pub use dash_block::DashBlock;
pub use falling_block::{FallStage, FallingBlock};
pub use jump_thru::JumpThru;
pub use solid::Solid;
pub use spikes::Spikes;
pub use spring::Spring;
pub use zipper::{ZipStage, Zipper};

// the parts of a room that change while she plays.
// spikes, springs and jump-throughs never change, so they stay on the room
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Entities {
    pub solids: Vec<Solid>,
    pub crystals: Vec<Crystal>,
    pub dash_blocks: Vec<DashBlock>,
    pub crumble_blocks: Vec<CrumbleBlock>,
    pub falling_blocks: Vec<FallingBlock>,
    pub zippers: Vec<Zipper>,
}
