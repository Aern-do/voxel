macro_rules! define_block {
    ($($(#[$attr:meta])? $block:ident: $visibility:ident),* $(,)?) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Block {
            $($(#[$attr])? $block),*
        }

        impl Block {
            pub fn visibility(self) -> Visibility {
                match self {
                    $(Self::$block => Visibility::$visibility),*
                }
            }

            pub fn texture_id(self) -> u32 {
                self as u32
            }
        }
    };
}
define_block!(
    Dirt: Opaque,
    Grass: Opaque,
    Water: Transparent,
    Sand: Opaque,
    Stone: Opaque,


    #[default]
    Air: Empty,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Visibility {
    Opaque,
    Transparent,
    Empty,
}
