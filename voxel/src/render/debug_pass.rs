use std::time::{Duration, Instant};

use voxel_util::Context;
use wgpu::RenderPass;
use wgpu_text::{
    glyph_brush::{
        ab_glyph::{FontRef, PxScale},
        OwnedSection, OwnedText,
    },
    BrushBuilder, TextBrush,
};

use crate::asset;

pub trait OwnedSectionExt {
    fn set_text<T: Into<String>>(&mut self, text: T) -> &mut OwnedText;
}

impl OwnedSectionExt for OwnedSection {
    fn set_text<T: Into<String>>(&mut self, text: T) -> &mut OwnedText {
        let text = OwnedText::new(text.into());

        match self.text.first() {
            Some(..) => self.text[0] = text,
            None => self.text.push(text),
        };

        &mut self.text[0]
    }
}

pub struct DebugPass {
    brush: TextBrush<FontRef<'static>>,

    fps_section: OwnedSection,
    last_fps_update: Instant,
}

impl DebugPass {
    pub fn new(context: &Context) -> Self {
        let brush = BrushBuilder::using_font_bytes(include_bytes!(asset!("monogram.ttf")))
            .expect("invalid font")
            .build(
                context.device(),
                context.config().width,
                context.config().height,
                context.config().format,
            );

        Self {
            brush,
            fps_section: OwnedSection::default().with_screen_position((5.0, 5.0)),
            last_fps_update: Instant::now(),
        }
    }

    pub fn update_fps(&mut self, delta_time: Duration) {
        if self.last_fps_update.elapsed() > Duration::from_millis(250) {
            let fps = 1.0 / delta_time.as_secs_f32();

            let text = self.fps_section.set_text(format!("FPS: {}", fps.round()));
            text.scale = PxScale::from(24.0);

            self.last_fps_update = Instant::now();
        }
    }

    pub fn update(&mut self, delta_time: Duration, context: &Context) {
        self.update_fps(delta_time);

        self.brush
            .queue(context.device(), context.queue(), [&self.fps_section])
            .expect("cache texture limit exceeded");
    }
}

impl DebugPass {
    pub fn draw<'r>(&'r self, render_pass: &mut RenderPass<'r>) {
        self.brush.draw(render_pass);
    }
}
