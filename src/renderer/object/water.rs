use crate::core::*;
use crate::renderer::*;
use std::sync::Arc;

const VERTICES_PER_SIDE: usize = 33;

#[derive(Clone, Copy, Debug)]
pub struct WaveParameters {
    /// The distance between each top of the wave.
    pub wavelength: f32,
    /// The distance from the top or bottom of the wave to the average water height.
    pub amplitude: f32,
    /// The speed at which the waves move.
    pub speed: f32,
}

impl Default for WaveParameters {
    fn default() -> Self {
        Self {
            wavelength: 1.0,
            amplitude: 0.01,
            speed: 0.5,
        }
    }
}

///
/// A water geometry with an applied material.
///
pub struct Water<M: Material> {
    patches: Vec<Gm<WaterPatch, M>>,
}
impl<M: Material + Clone> Water<M> {
    ///
    /// Constructs a new [Water] object with the given material and at the given height.
    ///
    /// The `center` is the center of the visualized water surface, which can be updated using [Self::set_center] to simualte an infinite water surface.
    /// The `side_length` is the length of one side of the visualized water surface.
    /// The `vertex_distance` is the distance between vertices.
    ///
    pub fn new(
        context: &Context,
        material: M,
        height: f32,
        center: Vec2,
        side_length: f32,
        vertex_distance: f32,
        parameters: [WaveParameters; 4],
    ) -> Self {
        let patch_size = vertex_distance * (VERTICES_PER_SIDE - 1) as f32;
        let patches_per_side = ((side_length / patch_size).ceil() as u32).max(1);
        let half_side_length = 0.5 * patches_per_side as f32 * patch_size;
        let index_buffer = Self::indices(context);
        let position_buffer = Self::positions(context, vertex_distance);
        let mut patches = Vec::new();
        for ix in 0..patches_per_side {
            for iy in 0..patches_per_side {
                let offset = vec2(
                    (ix as f32) * patch_size - half_side_length,
                    (iy as f32) * patch_size - half_side_length,
                );
                let patch = WaterPatch::new(
                    context,
                    vec3(center.x, height, center.y),
                    parameters,
                    offset,
                    vec2(patch_size, patch_size),
                    position_buffer.clone(),
                    index_buffer.clone(),
                );
                patches.push(Gm::new(patch, material.clone()));
            }
        }

        Self { patches }
    }

    ///
    /// Set the center of the water.
    /// To be able to move the water with the camera, thereby simulating infinite water.
    ///
    pub fn set_center(&mut self, center: Vec2) {
        self.patches.iter_mut().for_each(|m| {
            m.center.x = center.x;
            m.center.z = center.y;
        });
    }

    ///
    /// Set the average height of the water.
    ///
    pub fn set_height(&mut self, height: f32) {
        self.patches.iter_mut().for_each(|m| m.center.y = height);
    }

    ///
    /// Set the currently used [WaveParameters].
    ///
    pub fn set_parameters(&mut self, parameters: [WaveParameters; 4]) {
        self.patches
            .iter_mut()
            .for_each(|p| p.parameters = parameters);
    }

    ///
    /// For updating the animation. The time parameter should be some continious time, for example the time since start.
    ///
    pub fn update_animation(&mut self, time: f64) {
        self.patches.iter_mut().for_each(|m| m.time = time);
    }

    fn indices(context: &Context) -> Arc<ElementBuffer> {
        let mut indices: Vec<u32> = Vec::new();
        let stride = VERTICES_PER_SIDE as u32;
        let max = stride - 1;
        for r in 0..max {
            for c in 0..max {
                indices.push(r + c * stride);
                indices.push(r + 1 + c * stride);
                indices.push(r + (c + 1) * stride);
                indices.push(r + (c + 1) * stride);
                indices.push(r + 1 + c * stride);
                indices.push(r + 1 + (c + 1) * stride);
            }
        }
        Arc::new(ElementBuffer::new_with_data(context, &indices))
    }

    fn positions(context: &Context, vertex_distance: f32) -> Arc<VertexBuffer> {
        let mut data = vec![vec3(0.0, 0.0, 0.0); (VERTICES_PER_SIDE * VERTICES_PER_SIDE) as usize];
        for r in 0..VERTICES_PER_SIDE {
            for c in 0..VERTICES_PER_SIDE {
                let vertex_id = r * VERTICES_PER_SIDE + c;
                let x = r as f32 * vertex_distance;
                let z = c as f32 * vertex_distance;
                data[vertex_id as usize] = vec3(x, 0.0, z);
            }
        }
        Arc::new(VertexBuffer::new_with_data(context, &data))
    }
}

impl<'a, M: Material> IntoIterator for &'a Water<M> {
    type Item = &'a dyn Object;
    type IntoIter = std::vec::IntoIter<&'a dyn Object>;

    fn into_iter(self) -> Self::IntoIter {
        self.patches
            .iter()
            .map(|m| m as &dyn Object)
            .collect::<Vec<_>>()
            .into_iter()
    }
}

struct WaterPatch {
    context: Context,
    time: f64,
    center: Vec3,
    parameters: [WaveParameters; 4],
    offset: Vec2,
    size: Vec2,
    position_buffer: Arc<VertexBuffer>,
    index_buffer: Arc<ElementBuffer>,
}

impl WaterPatch {
    pub fn new(
        context: &Context,
        center: Vec3,
        parameters: [WaveParameters; 4],
        offset: Vec2,
        size: Vec2,
        position_buffer: Arc<VertexBuffer>,
        index_buffer: Arc<ElementBuffer>,
    ) -> Self {
        Self {
            context: context.clone(),
            time: 0.0,
            center,
            parameters,
            offset,
            size,
            position_buffer,
            index_buffer,
        }
    }
}

impl Geometry for WaterPatch {
    fn render_with_material(
        &self,
        material: &dyn Material,
        camera: &Camera,
        lights: &[&dyn Light],
    ) {
        let fragment_shader_source = material.fragment_shader_source(false, lights);
        self.context
            .program(
                &include_str!("shaders/water.vert"),
                &fragment_shader_source,
                |program| {
                    material.use_uniforms(program, camera, lights);
                    program.use_uniform(
                        "offset",
                        &self.center + vec3(self.offset.x, 0.0, self.offset.y),
                    );
                    program.use_uniform("viewProjection", camera.projection() * camera.view());
                    program.use_uniform("time", &(self.time as f32 * 0.001));
                    program.use_uniform_array(
                        "wavelengths",
                        &self
                            .parameters
                            .iter()
                            .map(|p| p.wavelength)
                            .collect::<Vec<_>>(),
                    );
                    program.use_uniform_array(
                        "amplitudes",
                        &self
                            .parameters
                            .iter()
                            .map(|p| p.amplitude)
                            .collect::<Vec<_>>(),
                    );
                    program.use_uniform_array(
                        "speed",
                        &self.parameters.iter().map(|p| p.speed).collect::<Vec<_>>(),
                    );
                    let render_states = RenderStates {
                        blend: Blend::TRANSPARENCY,
                        ..Default::default()
                    };

                    program.use_vertex_attribute("position", &self.position_buffer);

                    program.draw_elements(render_states, camera.viewport(), &self.index_buffer);
                },
            )
            .unwrap();
    }

    fn aabb(&self) -> AxisAlignedBoundingBox {
        let m = self
            .parameters
            .map(|p| p.amplitude)
            .into_iter()
            .reduce(f32::max)
            .unwrap();
        AxisAlignedBoundingBox::new_with_positions(&[
            self.center + vec3(self.offset.x, -m, self.offset.y),
            self.center + vec3(self.offset.x + self.size.x, m, self.offset.y + self.size.y),
        ])
    }
}
