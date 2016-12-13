#version 150

uniform uint n;
uniform vec2 window;

in vec2 vec;

void main() {
    // float x = 2.0 * gl_VertexID / n - 1.0;
    if (window.y > window.x) {
        gl_Position = vec4(vec.x, vec.y, 1.0, 1.0);
    } else {
        gl_Position = vec4(vec.y, vec.x, 1.0, 1.0);
    }
}
