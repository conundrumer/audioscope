precision mediump float;

uniform vec2 window;
uniform sampler2D sampleBuffer;

void main() {
    gl_FragColor = vec4(texture2D(sampleBuffer, (gl_FragCoord.xy - 0.5) / window).rb, 0.0, 1.0);
}
