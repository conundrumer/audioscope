precision mediump float;

uniform vec2 scale;
uniform sampler2D state;
uniform sampler2D samples;

void main() {
    if (gl_FragCoord.y > scale.y - 1.0) {
        // insert at top
        gl_FragColor = texture2D(samples, vec2(gl_FragCoord.x, scale.y) / scale);
    } else {
        // move down
        gl_FragColor = texture2D(state, (gl_FragCoord.xy + vec2(0.0, 1.0)) / scale);
    }
}
