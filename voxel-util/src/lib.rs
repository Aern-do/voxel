pub mod bind_group;
pub mod context;
pub mod render_pipeline;
pub mod sampler;
pub mod spritesheet;
pub mod texture;
pub mod uniform;

pub use bind_group::{AsBindGroup, Binding, BindingEntries, Fragment, ShaderResource, Vertex};
pub use context::Context;
pub use render_pipeline::{BasePipeline, ColorTargetStateExt, RenderPipelineBuilder, VertexLayout};
pub use sampler::Sampler;
pub use spritesheet::Spritesheet;
pub use texture::Texture;
pub use uniform::Uniform;

#[macro_export]
macro_rules! tuple_impl {
    ($generate_macro:ident; $($t:ident)*) => {
        tuple_impl!(@reverse $generate_macro; $($t)* @);
    };

    (@reverse $generate_macro:ident;) => {};
    (@reverse $generate_macro:ident; @ $($x:ident)*) => {};

    (@reverse $generate_macro:ident; $head:ident $($tail:ident)* @ $($xrev:ident)*) => {
        $generate_macro!($($xrev)* $head);
        tuple_impl!(@reverse $generate_macro; $($tail)* @ $($xrev)* $head);
    };
}

// https://veykril.github.io/tlborm/decl-macros/building-blocks/counting.html#recursion
#[macro_export]
macro_rules! count {
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $_f:tt $_g:tt $_h:tt $_i:tt $_j:tt
     $_k:tt $_l:tt $_m:tt $_n:tt $_o:tt
     $_p:tt $_q:tt $_r:tt $_s:tt $_t:tt
     $($tail:tt)*)
        => {20usize + count!($($tail)*)};
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $_f:tt $_g:tt $_h:tt $_i:tt $_j:tt
     $($tail:tt)*)
        => {10usize + count!($($tail)*)};
    ($_a:tt $_b:tt $_c:tt $_d:tt $_e:tt
     $($tail:tt)*)
        => {5usize + count!($($tail)*)};
    ($_a:tt
     $($tail:tt)*)
        => {1usize + count!($($tail)*)};
    () => {0usize};
}
