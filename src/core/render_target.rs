//!
//! Functionality for rendering to the screen or into textures.
//!
//!
mod screen;
#[doc(inline)]
pub use screen::*;

mod render_target2d;
#[doc(inline)]
pub use render_target2d::*;

mod render_target2d_array;
#[doc(inline)]
pub use render_target2d_array::*;

mod render_target_cube_map;
#[doc(inline)]
pub use render_target_cube_map::*;

use crate::context::consts;
use crate::core::*;

///
/// Defines which channels (red, green, blue, alpha and depth) to clear when starting to write to a
/// [RenderTarget] or the [Screen].
/// If `None` then the channel is not cleared and if `Some(value)` the channel is cleared to that value (the value must be between 0 and 1).
///
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ClearState {
    /// Defines the clear value for the red channel.
    pub red: Option<f32>,
    /// Defines the clear value for the green channel.
    pub green: Option<f32>,
    /// Defines the clear value for the blue channel.
    pub blue: Option<f32>,
    /// Defines the clear value for the alpha channel.
    pub alpha: Option<f32>,
    /// Defines the clear value for the depth channel. A value of 1 means a depth value equal to the far plane and 0 means a depth value equal to the near plane.
    pub depth: Option<f32>,
}

impl ClearState {
    ///
    /// Nothing will be cleared.
    ///
    pub const fn none() -> Self {
        Self {
            red: None,
            green: None,
            blue: None,
            alpha: None,
            depth: None,
        }
    }

    ///
    /// The depth will be cleared to the given value.
    ///
    pub const fn depth(depth: f32) -> Self {
        Self {
            red: None,
            green: None,
            blue: None,
            alpha: None,
            depth: Some(depth),
        }
    }

    ///
    /// The color channels (red, green, blue and alpha) will be cleared to the given values.
    ///
    pub const fn color(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red: Some(red),
            green: Some(green),
            blue: Some(blue),
            alpha: Some(alpha),
            depth: None,
        }
    }

    ///
    /// Both the color channels (red, green, blue and alpha) and depth will be cleared to the given values.
    ///
    pub const fn color_and_depth(red: f32, green: f32, blue: f32, alpha: f32, depth: f32) -> Self {
        Self {
            red: Some(red),
            green: Some(green),
            blue: Some(blue),
            alpha: Some(alpha),
            depth: Some(depth),
        }
    }
}

impl Default for ClearState {
    fn default() -> Self {
        Self::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0)
    }
}

///
/// The destination of applying a copy.
///
pub enum CopyDestination<'a, 'b, 'c, 'd, T: TextureDataType> {
    /// Copies to the [Screen].
    Screen,
    /// Copies to a [Texture2D].
    ColorTexture(&'d mut Texture2D<T>),
    /// Copies to a [DepthTargetTexture2D].
    DepthTexture(&'d mut DepthTargetTexture2D),
    /// Copies to a [RenderTarget].
    RenderTarget(&'c RenderTarget<'a, 'b, T>),
}

pub(in crate::core) fn new_framebuffer(
    context: &Context,
) -> ThreeDResult<crate::context::Framebuffer> {
    Ok(context
        .create_framebuffer()
        .ok_or(CoreError::RenderTargetCreation)?)
}

#[cfg(feature = "debug")]
fn check(context: &Context) -> ThreeDResult<()> {
    context
        .check_framebuffer_status()
        .or_else(|status| Err(CoreError::RenderTargetCreation))
}

fn clear(context: &Context, clear_state: &ClearState) {
    Program::set_write_mask(
        context,
        WriteMask {
            red: clear_state.red.is_some(),
            green: clear_state.green.is_some(),
            blue: clear_state.blue.is_some(),
            alpha: clear_state.alpha.is_some(),
            depth: clear_state.depth.is_some(),
        },
    );
    let clear_color = clear_state.red.is_some()
        || clear_state.green.is_some()
        || clear_state.blue.is_some()
        || clear_state.alpha.is_some();
    if clear_color {
        context.clear_color(
            clear_state.red.unwrap_or(0.0),
            clear_state.green.unwrap_or(0.0),
            clear_state.blue.unwrap_or(0.0),
            clear_state.alpha.unwrap_or(1.0),
        );
    }
    if let Some(depth) = clear_state.depth {
        context.clear_depth(depth);
    }
    context.clear(if clear_color && clear_state.depth.is_some() {
        consts::COLOR_BUFFER_BIT | consts::DEPTH_BUFFER_BIT
    } else {
        if clear_color {
            consts::COLOR_BUFFER_BIT
        } else {
            consts::DEPTH_BUFFER_BIT
        }
    });
}
