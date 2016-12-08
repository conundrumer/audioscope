#version 150

uniform vec2 window;

layout(lines_adjacency) in;
layout(triangle_strip, max_vertices = 4 ) out;

void main() {
    vec2 p1 = gl_in[1].gl_Position.xy * window;
    vec2 p2 = gl_in[2].gl_Position.xy * window;

    vec2 v1 = normalize( p2 - p1 );
    vec2 n1 = vec2( -v1.y, v1.x );

    gl_Position = vec4((p1 + n1 * 16.0) / window, 0.0, 1.0);
    EmitVertex();
    gl_Position = vec4((p1 - n1 * 16.0) / window, 0.0, 1.0);
    EmitVertex();
    gl_Position = vec4((p2 + n1 * 16.0) / window, 0.0, 1.0);
    EmitVertex();
    gl_Position = vec4((p2 - n1 * 16.0) / window, 0.0, 1.0);
    EmitVertex();

    EndPrimitive();
}
