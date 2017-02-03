precision mediump float;

varying float idx;

void main() {
    gl_FragColor = vec4(0.0, pow(1.0 / (1.0 + idx), 0.5), 0.0, 1.0);
}
