use smithay::{
    backend::renderer::{
        element::Kind,
        gles::{
            GlesPixelProgram, GlesRenderer, Uniform, UniformName, UniformType,
            element::PixelShaderElement,
        },
    },
    utils::{Logical, Rectangle},
};

const BORDER_SHADER: &str = r"
precision mediump float;
// The size or dimensions.
uniform vec2 u_resolution;
// Color of border.
uniform vec3 border_color;
// Thickness of border.
uniform float border_thickness;
// The ratio of the coordinate to the resolution.
varying vec2 v_coords;

void main() {
    // Get the pixel coordinates.
    vec2 coords = v_coords * u_resolution;

    // Step function is just (param1 < param2) return 1.0 for true and 0.0 for false.
    // On the left side, if the coordinate is less than the thickness, draw a border.
    float xl = step(coords.x, border_thickness);
    float yl = step(coords.y, border_thickness);
    // On the right side, if (coordinate - border_thickness) is less than the coordinate, draw a border.
    float xr = step(u_resolution.x - border_thickness, coords.x);
    float yr = step(u_resolution.y - border_thickness, coords.y);

    // The alpha will become 1.0 or greater if any of the above statements are true.
    float alpha = xl + yl + xr + yr;

    gl_FragColor = vec4(border_color * alpha, alpha);
}
";

// Определяем структуру которая хранит фрагментный шейдер.
// Эта структура будет хранится в контексте EGL рендеринга
pub struct BorderShader(pub GlesPixelProgram);

impl BorderShader {
    pub fn element(
        renderer: &GlesRenderer,
        geo: Rectangle<i32, Logical>,
        alpha: f32,
        border_color: u32,
    ) -> PixelShaderElement {
        // Retrieve shader from EGL rendering context.
        let program = renderer
            .egl_context()
            .user_data()
            .get::<BorderShader>()
            .unwrap()
            .0
            .clone();

        let point = geo.size.to_point();

        // Colors are 24 bits with 8 bits for each red, green, blue value.
        // To get each color, shift the bits over by the offset and zero
        // out the other colors. The bitwise AND 255 does this because it will
        // zero out everything but the last 8 bits. This is where the color
        // has been shifted to.
        let red = border_color >> 16 & 255;
        let green = border_color >> 8 & 255;
        let blue = border_color & 255;

        let border_thickness = 2.0;

        PixelShaderElement::new(
            program,
            geo,
            None,
            alpha,
            vec![
                Uniform::new("u_resolution", (point.x as f32, point.y as f32)),
                Uniform::new("border_color", (red as f32, green as f32, blue as f32)),
                Uniform::new("border_thickness", border_thickness),
            ],
            Kind::Unspecified,
        )
    }
}

pub fn compile_shaders(renderer: &mut GlesRenderer) {
    // Компилируем GLSL файл во фрагментный шейдер
    let border_shader = renderer
        .compile_custom_pixel_shader(
            BORDER_SHADER,
            &[
                UniformName::new("u_resolution", UniformType::_2f),
                UniformName::new("border_color", UniformType::_3f),
                UniformName::new("border_thickness", UniformType::_1f),
            ],
        )
        .unwrap();

    // Save pixel shader in EGL rendering context.
    renderer
        .egl_context()
        .user_data()
        .insert_if_missing(|| BorderShader(border_shader));
}
