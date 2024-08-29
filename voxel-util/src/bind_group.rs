use std::{marker::PhantomData, num::NonZeroU32, ops::Deref, sync::OnceLock};
use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BindingResource, BindingType,
    ShaderStages,
};

use crate::{context::Context, count, tuple_impl};
type SmallVec<T> = smallvec::SmallVec<[T; 8]>;

pub trait AsShaderStages {
    fn as_shader_stages() -> ShaderStages;
}

#[derive(Debug, Clone, Copy)]
pub struct Fragment;

impl AsShaderStages for Fragment {
    fn as_shader_stages() -> ShaderStages {
        ShaderStages::FRAGMENT
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vertex;

impl AsShaderStages for Vertex {
    fn as_shader_stages() -> ShaderStages {
        ShaderStages::VERTEX
    }
}

pub trait Binding {
    fn ty() -> BindingType;
    fn count() -> Option<NonZeroU32>;
    fn resource(&self) -> BindingResource;
}

#[derive(Debug)]
pub struct Layout<L: IntoLayout>(pub(crate) BindGroupLayout, pub(crate) PhantomData<L>);

impl<L: IntoLayout> Deref for Layout<L> {
    type Target = BindGroupLayout;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<L: IntoLayout> Layout<L> {
    pub fn erase(self) -> BindGroupLayout {
        self.0
    }
}

pub trait IntoLayout {
    type Bindings<'b>: IntoBindingResources
    where
        Self: 'b;
    fn into_binding_entries() -> &'static [BindGroupLayoutEntry];
}

pub trait IntoBindingResources {
    fn into_binding_resources(&self) -> SmallVec<BindGroupEntry>;
}

#[derive(Debug)]
pub struct ShaderResource {
    bind_group: BindGroup,
    layout: BindGroupLayout,
}

impl ShaderResource {
    pub(crate) fn new(bind_group: BindGroup, layout: BindGroupLayout) -> Self {
        Self { bind_group, layout }
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn layout(&self) -> &BindGroupLayout {
        &self.layout
    }
}

pub trait AsBindGroup {
    type Layout: IntoLayout;

    fn resources(&self) -> <Self::Layout as IntoLayout>::Bindings<'_>;

    fn as_bind_group(&self, layout: &Layout<Self::Layout>, context: &Context) -> BindGroup {
        context.create_bind_group(layout, self.resources())
    }

    fn as_bind_group_layout(context: &Context) -> Layout<Self::Layout> {
        context.create_bind_group_layout::<Self::Layout>()
    }

    fn as_shader_resource(&self, context: &Context) -> ShaderResource {
        let layout = Self::as_bind_group_layout(context);
        let bind_group = self.as_bind_group(&layout, context);

        ShaderResource {
            bind_group,
            layout: layout.erase(),
        }
    }
}

macro_rules! impl_into_binding_entries {
    ($($generic:ident)*) => {paste::paste!{
        impl<$([<$generic S>]: AsShaderStages, [<$generic B>]: Binding),*> IntoLayout for ($(([<$generic S>], [<$generic B>])),*, ) {
            type Bindings<'b> = ($(&'b [<$generic B>]),*,) where Self: 'b;

            fn into_binding_entries() -> &'static [BindGroupLayoutEntry] {
                static LOCK: OnceLock<[BindGroupLayoutEntry; { count!($($generic)*) }]> = OnceLock::new();

                let mut index = 0;
                LOCK.get_or_init(|| {
                    [$(BindGroupLayoutEntry {
                    binding: {
                        let binding = index;
                        index += 1;
                        binding
                    },
                    ty: [<$generic B>]::ty(),
                    count: [<$generic B>]::count(),
                    visibility: [<$generic S>]::as_shader_stages()
                    }),*]
                })
            }
        }

        impl<'b, $($generic: Binding),*> IntoBindingResources for ($(&'b $generic),*, ) {
            fn into_binding_resources(&self) -> SmallVec<BindGroupEntry> {
                let ($([<$generic:lower>]),*,) = self;
                let mut index = 0;

                #[allow(unused_assignments)]
                SmallVec::from_iter([$(BindGroupEntry {
                    binding: {
                        let binding = index;
                        index += 1;
                        binding
                    },
                    resource: [<$generic:lower>].resource(),
                }),*])
            }
        }
    }};

}

tuple_impl!(impl_into_binding_entries; A B C D E F G H I J K L);

impl<AS: AsShaderStages, AB: Binding> IntoLayout for (AS, AB) {
    type Bindings<'b> = &'b AB
    where
        Self: 'b;

    fn into_binding_entries() -> &'static [BindGroupLayoutEntry] {
        static LOCK: OnceLock<[BindGroupLayoutEntry; 1]> = OnceLock::new();

        LOCK.get_or_init(|| {
            [BindGroupLayoutEntry {
                binding: 0,
                visibility: AS::as_shader_stages(),
                ty: AB::ty(),
                count: AB::count(),
            }]
        })
    }
}

impl<'b, A: Binding> IntoBindingResources for &'b A {
    fn into_binding_resources(&self) -> SmallVec<BindGroupEntry> {
        SmallVec::from_iter([BindGroupEntry {
            binding: 0,
            resource: self.resource(),
        }])
    }
}