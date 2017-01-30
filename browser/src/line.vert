attribute vec2 position;

void main() {
    gl_Position = vec4(position.x * 2.0 - 1.0, position.y, 0.0, 1.0);
}
