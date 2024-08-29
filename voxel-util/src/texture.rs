use std::num::NonZero;

use image::RgbaImage;
use wgpu::{
    BindingResource, BindingType, Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d,
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
};

use crate::{Binding, Context};

#[derive(Debug, Clone, Copy)]
pub struct TextureData<'d> {
    data: &'d [u8],
    size: (u32, u32),
    format: TextureFormat,
}

impl<'d> TextureData<'d> {
    pub fn new(data: &'d [u8], size @ (width, height): (u32, u32), format: TextureFormat) -> Self {
        let block_copy_size = format
            .block_copy_size(None)
            .expect("unknown block copy size");

        assert!((width * block_copy_size) * height == data.len() as u32);

        Self { data, size, format }
    }
}

impl<'d> From<&'d RgbaImage> for TextureData<'d> {
    fn from(image: &'d RgbaImage) -> Self {
        Self {
            data: &image,
            size: image.dimensions(),
            format: TextureFormat::Rgba8UnormSrgb,
        }
    }
}

#[derive(Debug)]
pub struct Texture {
    texture: wgpu::Texture,
    view: TextureView,
    size: (u32, u32),

    format: TextureFormat,
}

impl Texture {
    pub fn new(
        size @ (width, height): (u32, u32),
        usage: TextureUsages,
        format: TextureFormat,
        context: &Context,
    ) -> Self {
        let texture = context.device().create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: width,
                height: height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: usage,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        Self {
            texture,
            view,
            size,
            format,
        }
    }

    pub fn from_data<'d, D>(data: D, usage: TextureUsages, context: &Context) -> Self
    where
        TextureData<'d>: From<D>,
    {
        let texture_data = TextureData::from(data);
        let texture = Self::new(texture_data.size, usage, texture_data.format, context);
        texture.upload_data::<TextureData>(texture_data, context);

        texture
    }

    pub fn upload_data<'d, D>(&self, texture_data: D, context: &Context)
    where
        TextureData<'d>: From<D>,
    {
        let texture_data = TextureData::from(texture_data);
        let (width, height) = texture_data.size;

        self.upload_data_into_region::<TextureData>(texture_data, (0, 0, width, height), context)
    }

    pub fn upload_data_into_region<'d, D>(
        &self,
        texture_data: D,
        (min_x, min_y, max_x, max_y): (u32, u32, u32, u32),
        context: &Context,
    ) where
        TextureData<'d>: From<D>,
    {
        let texture_data = TextureData::from(texture_data);

        let (width, height) = texture_data.size;
        let (texture_width, texture_height) = self.size;
        let (region_width, region_height) = (max_x - min_x, max_y - min_y);

        assert!(region_width <= width && region_height <= height);
        assert!(width <= texture_width && height <= texture_height);
        assert!(texture_data.format == self.format);

        let block_copy_size = self
            .format
            .block_copy_size(None)
            .expect("unknown block copy size");

        context.queue().write_texture(
            ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d {
                    x: min_x,
                    y: min_y,
                    z: 0,
                },
                aspect: TextureAspect::All,
            },
            texture_data.data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(block_copy_size * region_width),
                rows_per_image: Some(region_height),
            },
            Extent3d {
                width: region_width,
                height: region_height,
                depth_or_array_layers: 1,
            },
        )
    }

    pub fn view(&self) -> &TextureView {
        &self.view
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }
}

impl Binding for Texture {
    fn ty() -> BindingType {
        BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable: true },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        }
    }

    fn count() -> Option<NonZero<u32>> {
        None
    }

    fn resource(&self) -> BindingResource {
        BindingResource::TextureView(&self.view)
    }
}
