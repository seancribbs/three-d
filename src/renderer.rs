//!
//! High-level features for easy loading and rendering of different types of objects with different types of shading.
//! Can be combined seamlessly with the mid-level features in the `core` module and also with calls in the `context` module as long as the graphics state is reset.
//!

pub use crate::core::*;

mod camera;
#[doc(inline)]
pub use camera::*;

mod shading;
#[doc(inline)]
pub use shading::*;

mod gui;
#[doc(inline)]
pub use gui::*;

pub mod effect;
pub use effect::*;

pub mod io;
pub use io::*;

pub mod light;
pub use light::*;

pub mod object;
pub use object::*;

pub fn ray_intersect(
    context: &Context,
    position: Vec3,
    direction: Vec3,
    max_depth: f32,
    geometries: &[&dyn Geometry],
) -> Result<Option<Vec3>, Error> {
    let viewport = Viewport::new_at_origo(1, 1);
    let up = if direction.dot(vec3(1.0, 0.0, 0.0)).abs() > 0.99 {
        direction.cross(vec3(0.0, 1.0, 0.0))
    } else {
        direction.cross(vec3(1.0, 0.0, 0.0))
    };
    let camera = Camera::new_orthographic(
        context,
        viewport,
        position,
        position + direction * max_depth,
        up,
        0.01,
        0.0,
        max_depth,
    )?;
    let texture = ColorTargetTexture2D::<f32>::new(
        context,
        viewport.width,
        viewport.height,
        Interpolation::Nearest,
        Interpolation::Nearest,
        None,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
        Format::RGBA,
    )?;
    let depth_texture = DepthTargetTexture2D::new(
        context,
        viewport.width,
        viewport.height,
        Wrapping::ClampToEdge,
        Wrapping::ClampToEdge,
        DepthFormat::Depth32F,
    )?;
    let render_target = RenderTarget::new(context, &texture, &depth_texture)?;

    let render_states = RenderStates {
        write_mask: WriteMask {
            red: true,
            depth: true,
            ..WriteMask::NONE
        },
        depth_test: DepthTestType::Less,
        ..Default::default()
    };
    render_target.write(
        ClearState {
            red: Some(1.0),
            depth: Some(1.0),
            ..ClearState::none()
        },
        || {
            for geometry in geometries {
                if geometry
                    .aabb()
                    .map(|aabb| camera.in_frustum(&aabb))
                    .unwrap_or(true)
                {
                    geometry.render_depth_to_red(render_states, &camera, max_depth)?;
                }
            }
            Ok(())
        },
    )?;
    let depth = texture.read(viewport)?[0];
    Ok(if depth < 1.0 {
        Some(position + direction * depth * max_depth)
    } else {
        None
    })
}
