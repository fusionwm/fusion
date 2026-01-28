struct InstanceData {
    uv: vec4<f32>,
    size: vec2<f32>,
    _padding0: vec2<f32>,

    model_matrix_0: vec4<f32>,
    model_matrix_1: vec4<f32>,
    model_matrix_2: vec4<f32>,
    model_matrix_3: vec4<f32>,
    color: vec4<f32>,

    stroke_color_left: vec4<f32>,
    stroke_color_right: vec4<f32>,
    stroke_color_top: vec4<f32>,
    stroke_color_bottom: vec4<f32>,

    color_end: vec4<f32>,

    corner_radii: vec4<f32>,

    //degree: f32,
    //use_gradient: u32,
    //support_stroke: u32,
    //stroke_width: f32,
    misc: vec4<f32>,
};

@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var t_sampler: sampler;
// Наш новый Storage Buffer
@group(0) @binding(2) var<storage, read> instances: array<InstanceData>;

struct Vertex {
    @location(0) position: vec3<f32>,
};

struct VertexPayload {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) local_pos: vec2<f32>, // Для расчета SDF (в пикселях)
    @location(2) instance_id: u32,     // Прокидываем ID, чтобы читать данные во фрагментнике
};

@vertex
fn vs_main(
    @builtin(instance_index) instance_id: u32,
    vertex: Vertex
) -> VertexPayload {
    let instance = instances[instance_id];
    var out: VertexPayload;

    // UV на основе прямоугольника из буфера
    out.uv = vec2<f32>(
        mix(instance.uv.x, instance.uv.z, vertex.position.x),
        mix(instance.uv.y, instance.uv.w, vertex.position.y)
    );

    let model = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    // Локальная позиция от центра (для SDF)
    out.local_pos = (vertex.position.xy - 0.5) * instance.size;
    out.position = model * vec4<f32>(vertex.position, 1.0);
    out.instance_id = instance_id;
    return out;
}

fn sd_rounded_box(p: vec2<f32>, b: vec2<f32>, r: vec4<f32>) -> f32 {
    var rad = r;
    // Выбор радиуса в зависимости от квадранта
    if (p.x < 0.0 && p.y > 0.0) { rad.x = rad.x; }      // Top-Left
    else if (p.x > 0.0 && p.y > 0.0) { rad.x = rad.y; } // Top-Right
    else if (p.x > 0.0 && p.y < 0.0) { rad.x = rad.z; } // Bottom-Right
    else { rad.x = rad.w; }                            // Bottom-Left

    let q = abs(p) - b + rad.x;
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - rad.x;
}



fn aa_step(dist: f32) -> f32 {
    // 1. Вычисляем градиент (скорость изменения расстояния на пиксель)
    // Используем длину вектора производных для стабильности при поворотах
    let der = fwidth(dist);

    // 2. Аналитическое сглаживание:
    // Вместо простого деления, используем смещение на полпикселя.
    // Это гарантирует, что линия 1px будет иметь идеальную плотность.
    return clamp(0.5 - dist / der, 0.0, 1.0);
}

@fragment
fn fs_main(in: VertexPayload) -> @location(0) vec4<f32> {
    let instance = instances[in.instance_id];

    // --- 1. ПРОВЕРКА ДИСТАНЦИИ (SDF) ---
    let half_size = instance.size * 0.5;
    let dist = sd_rounded_box(in.local_pos, half_size, instance.corner_radii);

    // Применяем идеальное сглаживание
    let smoothness = aa_step(dist);

    // Ранний выход для пустых пикселей
    if (smoothness <= 0.0) { discard; }

    // --- 2. ЦВЕТ И ТЕКСТУРА ---
    let texColor = textureSample(texture, t_sampler, in.uv);
    var baseColor = texColor * instance.color;

    // --- 3. ГРАДИЕНТ (Оптимизированный) ---
    if instance.misc.y >= 1.0 {
        let angle = instance.misc.x * 0.01745329; // Предрассчитанный PI/180
        let dir = vec2<f32>(cos(angle), sin(angle));
        // Используем local_pos для более точного градиента, чем UV
        let t = clamp(dot(in.local_pos / instance.size, dir) + 0.5, 0.0, 1.0);
        baseColor = texColor * mix(instance.color, instance.color_end, t);
    }

    // --- 4. ОБВОДКА (Stroke) ---
    var final_color = baseColor;
    let stroke_width = instance.misc.w;

    if stroke_width > 0.0 && instance.misc.z >= 1.0 {
        // Внутренняя граница обводки
        let stroke_inner_dist = dist + stroke_width;
        let stroke_alpha = aa_step(stroke_inner_dist);

        // Выбор цвета (используем плавное смешивание для углов)
        var s_color = instance.stroke_color_left;
        let stroke_norm = stroke_width / instance.size; // vec2

        // Улучшенная логика сторон: используем более точные пороги
        if (in.uv.x > (1.0 - stroke_norm.x)) { s_color = instance.stroke_color_right; }
        else if (in.uv.y > (1.0 - stroke_norm.y)) { s_color = instance.stroke_color_bottom; }
        else if (in.uv.y < stroke_norm.y) { s_color = instance.stroke_color_top; }

        // Смешиваем обводку с основным цветом
        final_color = mix(s_color, baseColor, stroke_alpha);
    }

    // Финальный результат с учетом альфа-канала и сглаживания
    return vec4<f32>(final_color.rgb, final_color.a * smoothness);
}

/*
@fragment
fn fs_main(in: VertexPayload) -> @location(0) vec4<f32> {
    let instance = instances[in.instance_id];
    let texColor = textureSample(texture, t_sampler, in.uv);
    var baseColor = texColor * instance.color;

    let degree: f32 = instance.misc.x;
    let use_gradient: u32 = u32(instance.misc.y);
    let support_stroke: u32 = u32(instance.misc.z);
    let stroke_width: f32 = instance.misc.w;

    //if use_gradient >= 1u {
    //    let angle = degree * 3.14159265 / 180.0;
    //    let dir = vec2<f32>(cos(angle), sin(angle));
    //    let centered_uv = in.uv - vec2<f32>(0.5);
    //    let max_len = 0.707;
    //    let raw_t = dot(centered_uv, dir);
    //    let t = clamp((raw_t / max_len + 1.0) * 0.5, 0.0, 1.0);
    //    baseColor = texColor * mix(instance.color, instance.color_end, t);
    //}

    //if stroke_width > 0.0 && support_stroke >= 1u {
    //    let stroke_norm = vec2(
    //        stroke_width / instance.size.x,
    //        stroke_width / instance.size.y
    //    );

    //    let left_factor = step(in.uv.x, stroke_norm.x);
    //    let right_factor = step(1.0 - in.uv.x, stroke_norm.x);
    //    let top_factor = step(in.uv.y, stroke_norm.y);
    //    let bottom_factor = step(1.0 - in.uv.y, stroke_norm.y);

    //    if (left_factor > 0.0) {
    //        return instance.stroke_color_left;
    //    } else if (right_factor > 0.0) {
    //        return instance.stroke_color_right;
    //    } else if (top_factor > 0.0) {
    //        return instance.stroke_color_top;
    //    } else if (bottom_factor > 0.0) {
    //        return instance.stroke_color_bottom;
    //    }
    //}

    return baseColor;
}
*/
