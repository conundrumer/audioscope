#version 150

uniform bool colorize;
uniform float base_hue;

in float relative_length;
in float angle;

out vec4 color;

// https://github.com/hughsk/glsl-hsv2rgb
vec3 hsv2rgb(vec3 c) {
  vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main() {
    if (colorize) {
        // increase/decrease length by factor of 2 (an octave) -> same hue
        float phase = log2(relative_length);
        color = vec4(hsv2rgb(vec3(base_hue + phase, 1.0, 1.0)), 1.0);
    } else {
        color = vec4(hsv2rgb(vec3(base_hue, 1.0, 1.0)), 1.0);
    }
}
