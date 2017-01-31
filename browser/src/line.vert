precision mediump float;

uniform sampler2D samples;
uniform vec2 scale;

attribute float index;

// varying float signed_dist;

float decode(vec2 c) {
    return float(c.x * 255.0 * 256.0 + c.y * 255.0) / (256.0 * 256.0 - 1.0);
}

const float t_max = 0.01;
const float t_min = 0.0001;
const float t_flat = 0.0005;

void main() {
    float i = floor(index / 4.0);
    int j = int(index - 4.0 * i + 0.5);

    vec4 sample = texture2D(samples, vec2(i, 0.0) / scale);
    vec4 prev_sample = texture2D(samples, vec2(i - 1.0, 0.0) / scale);
    vec4 next_sample = texture2D(samples, vec2(i + 1.0, 0.0) / scale);

    vec2 pos = vec2(decode(sample.rg), decode(sample.ba));
    vec2 prev_pos = vec2(decode(prev_sample.rg), decode(prev_sample.ba));
    vec2 next_pos = vec2(decode(next_sample.rg), decode(next_sample.ba));

    float prev_len = distance(pos, prev_pos);
    float next_len = distance(pos, next_pos);
    float avg_len = mix(prev_len, next_len, 0.5);

    float thickness = (t_max - t_min) * t_flat / (t_flat + avg_len) + t_min;

    vec2 delta = vec2(0.0, 0.0);
    if (j == 0) {
        delta = pos - prev_pos;
        // signed_dist = thickness;
    } else if (j == 1) {
        delta = prev_pos - pos;
        // signed_dist = -thickness;
    } else if (j == 2) {
        delta = next_pos - pos;
        // signed_dist = thickness;
    } else if (j == 3) {
        delta = pos - next_pos;
        // signed_dist = -thickness;
    }

    pos = pos + thickness * normalize(vec2(-delta.y, delta.x));

    gl_Position = vec4(pos * 1.8 - 0.9, 0.0, 1.0);
}
