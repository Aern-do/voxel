use glam::Vec3;

use super::block_face::{BlockFace, Direction};

macro_rules! define_block {
    ($($variant_name:ident $(($visibility:ident))?: $texture_id:literal),* $(,)?) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Block {
            #[default]
            Air,
            $($variant_name),*
        }

        impl Block {
            pub fn visibility(&self) -> Visibility {
                match self {
                    Self::Air => Visibility::Empty,
                    $(Self::$variant_name => define_block!(@visibility $($visibility)?)),*,
                }
            }

            pub fn texture_id(&self) -> u32 {
                match self {
                    Self::Air => unreachable!(),
                    $(Self::$variant_name => $texture_id),*,
                }
            }

            pub fn is_opaque(&self) -> bool {
                matches!(self.visibility(), Visibility::Opaque)
            }

            pub fn is_transparent(&self) -> bool {
                matches!(self.visibility(), Visibility::Transparent)
            }

            pub fn is_empty(&self) -> bool {
                matches!(self.visibility(), Visibility::Empty)

            }
        }
    };


    (@visibility transparent) => {
        Visibility::Transparent
    };

    (@visibility) => {
        Visibility::Opaque
    }
}

define_block! {
    Grass: 0,
    Water(transparent): 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Visibility {
    Opaque,
    Transparent,
    Empty,
}
