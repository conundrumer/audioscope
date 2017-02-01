precision mediump float;

uniform sampler2D samples;
uniform vec2 scale;

attribute float index;

float decode(vec2 c) {
    return (c.x * 255.0 * 256.0 + c.y * 255.0) / (256.0 * 256.0 - 1.0);
}

vec2 get_sample(int i) {
    vec4 sample = texture2D(samples, vec2(i, 0.0) / scale);
    return vec2(decode(sample.rg), decode(sample.ba));
}

const float t_max = 0.01;
const float t_min = 0.0001;
const float t_flat = 0.0005;

void main() {
    int i = int(index) / 4;
    int j = int(index) - 4 * i;

    vec2 pos = get_sample(i);
    vec2 prev_pos = get_sample(i - 1);
    vec2 next_pos = get_sample(i + 1);

    float prev_len = distance(pos, prev_pos);
    float next_len = distance(pos, next_pos);
    float avg_len = mix(prev_len, next_len, 0.5);

    float thickness = (t_max - t_min) * t_flat / (t_flat + avg_len) + t_min;

    vec2 delta = vec2(0.0, 0.0);
    if (j == 0) {
        delta = pos - prev_pos;
    } else if (j == 1) {
        delta = prev_pos - pos;
    } else if (j == 2) {
        delta = next_pos - pos;
    } else if (j == 3) {
        delta = pos - next_pos;
    }

    pos = pos + thickness * normalize(vec2(-delta.y, delta.x));

    gl_Position = vec4(pos * 2.0 - 1.0, 0.0, 1.0);
}
