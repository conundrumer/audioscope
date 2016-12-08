#version 150

uniform float n;

in float v;

void main() {
    float x = 2.0 * gl_VertexID / n - 1.0;
    float y = v;
    gl_Position = vec4(x, y, 1.0, 1.0);
}
