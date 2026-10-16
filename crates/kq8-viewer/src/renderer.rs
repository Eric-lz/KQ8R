//! `--renderer auto|vulkan|dx12`: which graphics API Bevy draws with
//! (docs/DESIGN.md, "choosing the graphics API"). Our own option, not
//! from the original game.
//!
//! Bevy draws through wgpu, which can talk to Vulkan, DirectX 12, Metal
//! or OpenGL. `auto` lets wgpu choose (and keeps the `WGPU_BACKEND`
//! environment variable working); the others allow only that API.

use bevy::prelude::*;
use bevy::render::renderer::RenderAdapterInfo;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::render::RenderPlugin;

use crate::Shared;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Renderer {
    Auto,
    Vulkan,
    Dx12,
}

impl Renderer {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "vulkan" | "vk" => Ok(Self::Vulkan),
            "dx12" | "d3d12" | "directx" => Ok(Self::Dx12),
            // Bevy is built without its OpenGL part (feature `gles`), and
            // on Windows OpenGL would also need the ANGLE library.
            "gl" | "opengl" | "gles" => Err("--renderer gl is not available: this viewer is built without OpenGL; use auto, vulkan or dx12".into()),
            v => Err(format!("--renderer needs auto, vulkan or dx12, not '{v}'")),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Vulkan => "vulkan",
            Self::Dx12 => "dx12",
        }
    }

    /// Bevy's render plugin for this choice. `Auto` is Bevy's default
    /// (which reads `WGPU_BACKEND`); the others override it.
    pub fn plugin(self) -> RenderPlugin {
        let backends = match self {
            Self::Auto => return RenderPlugin::default(),
            Self::Vulkan => Backends::VULKAN,
            Self::Dx12 => Backends::DX12,
        };
        RenderPlugin {
            render_creation: RenderCreation::from(WgpuSettings { backends: Some(backends), ..default() }),
            ..default()
        }
    }
}

/// What was asked for, kept for the log line once the card is set up.
#[derive(Resource)]
pub struct RendererChoice(pub Renderer);

/// Writes once to `viewer.log` which API, card and driver are in use. The
/// card is set up while the app starts, so this waits until its details
/// appear.
pub fn report_renderer(info: Option<Res<RenderAdapterInfo>>, choice: Res<RendererChoice>, shared: Res<Shared>, mut done: Local<bool>) {
    let Some(info) = info.filter(|_| !*done) else { return };
    *done = true;
    shared.log.lock().unwrap().summary(format!(
        "renderer: asked {} got {:?} on \"{}\" (driver {} {})",
        choice.0.name(),
        info.backend,
        info.name,
        info.driver,
        info.driver_info
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_names() {
        assert_eq!(Renderer::parse("auto"), Ok(Renderer::Auto));
        assert_eq!(Renderer::parse("Vulkan"), Ok(Renderer::Vulkan));
        assert_eq!(Renderer::parse("vk"), Ok(Renderer::Vulkan));
        assert_eq!(Renderer::parse("DX12"), Ok(Renderer::Dx12));
        assert_eq!(Renderer::parse("d3d12"), Ok(Renderer::Dx12));
        assert!(Renderer::parse("gl").is_err());
        assert!(Renderer::parse("metal").is_err());
    }
}
