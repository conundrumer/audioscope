#version 150

in float x;
in float y;

void main() {
    // gl_VertexID
    gl_Position = vec4(x, y, 1.0, 1.0);
}
