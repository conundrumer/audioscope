precision mediump float;

uniform sampler2D samples;
uniform vec2 scale;

attribute float index;

float decode(vec2 c) {
    return float(c.x * 255.0 * 256.0 + c.y * 255.0) / (256.0 * 256.0 - 1.0);
}

void main() {
    vec4 sample = texture2D(samples, vec2(index, 0.0) / scale);
    vec2 position = vec2(decode(sample.rg), decode(sample.ba));

    gl_Position = vec4(position * 2.0 - 1.0, 0.0, 1.0);
}
