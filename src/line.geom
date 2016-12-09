#version 150

uniform uint n;
uniform vec2 window;
uniform float thickness;
uniform float thinning;

layout(lines_adjacency) in;
layout(triangle_strip, max_vertices = 5) out;

out float relative_length;

void emit_position(vec2 pos) {
    gl_Position = vec4(pos / window, 0.0, 1.0);
    EmitVertex();
}

// heavily based on paul houx's miter polylines
// https://github.com/paulhoux/Cinder-Samples/blob/master/GeometryShader/assets/shaders/lines2.geom
void main() {
    // get the four vertices passed to the shader:
    vec2 p0_ = gl_in[0].gl_Position.xy;
    vec2 p1_ = gl_in[1].gl_Position.xy;
    vec2 p2_ = gl_in[2].gl_Position.xy;
    vec2 p3_ = gl_in[3].gl_Position.xy;

    // relative length of first two segments (NOT in terms of screen coordinates)
    float length_a_ = distance(p0_, p1_);
    float length_b_ = distance(p1_, p2_);

    // transform to screen coordinates
    vec2 p0 = p0_ * window;
    vec2 p1 = p1_ * window;
    vec2 p2 = p2_ * window;
    vec2 p3 = p3_ * window;

    // vectors for the 3 segments (previous, current, next)
    vec2 v0 = p1 - p0;
    vec2 v1 = p2 - p1;
    vec2 v2 = p3 - p2;

    // the normal of each of the 3 segments (previous, current, next)
    vec2 n0 = normalize(vec2(-v0.y, v0.x));
    vec2 n1 = normalize(vec2(-v1.y, v1.x));
    vec2 n2 = normalize(vec2(-v2.y, v2.x));

    // miter lines by averaging the normals of the 2 segments
    vec2 miter_a_norm = normalize(n0 + n1);    // miter at start of current segment
    vec2 miter_b_norm = normalize(n1 + n2);    // miter at end of current segment

    // thicknesses at p1 and p2
    float thickness_adjusted = thickness * mix(1.0, 4.0, thinning);
    float thickness_a = min(thickness, thickness_adjusted / mix(1.0, n * length_a_, thinning));
    float thickness_b = min(thickness, thickness_adjusted / mix(1.0, n * length_b_, thinning));

    // the length of the miter by projecting it onto normal and then inverse it
    vec2 miter_a = miter_a_norm * thickness_a / dot(miter_a_norm, n1);
    vec2 miter_b = miter_b_norm * thickness_b / dot(miter_b_norm, n1);

    relative_length = length_a_;
    if(dot(v0, n1) > 0) {
        // start at negative miter
        emit_position(p1 - miter_a);
        // proceed to positive normal
        emit_position(p1 + thickness_a * n1);
    } else {
        // start at negative normal
        emit_position(p1 - thickness_a * n1);
        // proceed to positive miter
        emit_position(p1 + miter_a);
    }

    relative_length = length_b_;
    if (dot(v2, n1) < 0) {
        // proceed to negative miter
        emit_position(p2 - miter_b);
        // proceed to positive normal
        emit_position(p2 + thickness_b * n1);
        // end at positive normal
        emit_position(p2 + thickness_b * n2);
    } else {
        // proceed to negative normal
        emit_position(p2 - thickness_b * n1);
        // proceed to positive miter
        emit_position(p2 + miter_b);
        // end at negative normal
        emit_position(p2 - thickness_b * n2);
    }

    EndPrimitive();
}
