use crate::core::*;
use crate::renderer::*;

///
/// Forward render pipeline which can render objects (implementing the [Object] trait) with materials (implementing the [ForwardMaterial] trait) and lighting.
/// Forward rendering directly draws to the given render target (for example the screen) and is therefore the same as calling [Object::render_forward] directly.
///
pub struct ForwardPipeline {
    context: Context,
}

impl ForwardPipeline {
    ///
    /// Constructor.
    ///
    pub fn new(context: &Context) -> Result<Self> {
        Ok(Self {
            context: context.clone(),
        })
    }

    ///
    /// Render the objects with the given surface materials and the given set of lights.
    /// Must be called in a render target render function, for example in the callback function of [Screen::write].
    ///
    #[deprecated = "Use render_pass instead"]
    pub fn light_pass(
        &self,
        camera: &Camera,
        objects: &[(&dyn Object, &PhysicalMaterial)],
        ambient_light: Option<&AmbientLight>,
        directional_lights: &[&DirectionalLight],
        spot_lights: &[&SpotLight],
        point_lights: &[&PointLight],
    ) -> Result<()> {
        let mut lights = Vec::new();
        if let Some(light) = ambient_light {
            lights.push(light as &dyn Light)
        }
        for light in directional_lights {
            lights.push(*light as &dyn Light);
        }
        for light in spot_lights {
            lights.push(*light as &dyn Light);
        }
        for light in point_lights {
            lights.push(*light as &dyn Light);
        }
        let objects = objects
            .iter()
            .map(|(obj, mat)| {
                (
                    *obj,
                    LitMaterial {
                        material: &mat,
                        lights: &lights,
                    },
                )
            })
            .collect::<Vec<_>>();
        self.render_pass(
            camera,
            &objects
                .iter()
                .map(|(obj, mat)| (*obj, mat as &dyn ForwardMaterial))
                .collect::<Vec<_>>(),
        )
    }

    pub fn render_pass(
        &self,
        camera: &Camera,
        objects: &[(&dyn Object, &dyn ForwardMaterial)],
    ) -> Result<()> {
        for (object, material) in objects {
            if in_frustum(camera, *object) {
                object.render_forward(*material, camera)?;
            }
        }

        Ok(())
    }

    pub fn depth_pass(&self, camera: &Camera, objects: &[&dyn Object]) -> Result<()> {
        let depth_material = DepthMaterial {
            render_states: RenderStates {
                write_mask: WriteMask::DEPTH,
                ..Default::default()
            },
            ..Default::default()
        };
        for object in objects {
            if in_frustum(camera, *object) {
                object.render_forward(&depth_material, camera)?;
            }
        }
        Ok(())
    }

    pub fn depth_pass_texture(
        &self,
        camera: &Camera,
        objects: &[&dyn Object],
    ) -> Result<DepthTargetTexture2D> {
        let depth_texture = DepthTargetTexture2D::new(
            &self.context,
            camera.viewport().width,
            camera.viewport().height,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
            DepthFormat::Depth32F,
        )?;
        depth_texture.write(Some(1.0), || self.depth_pass(&camera, objects))?;
        Ok(depth_texture)
    }
}
